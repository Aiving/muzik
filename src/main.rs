#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::unreadable_literal
)]

mod mpris;
pub mod muzui;

use indexable::Indexable;
use material_colors::{
    color::Argb, dynamic_color::variant::Variant, image::ImageReader, theme::ThemeBuilder,
};
use mpris::{generated::player::Metadata, Player};
use muzui::{
    language::{lexer::Lexer, parser::Parser, program::parse_node},
    layout::{Length, Operation},
    node::Node,
    RenderContext,
};
use rusfit::rusfit;
use serde::{Deserialize, Deserializer, Serialize};
use skia_safe::{
    canvas::SaveLayerRec, surfaces, svg::Dom, wrapper::PointerWrapper, BlendMode, BlurStyle, Color,
    /* Color4f,  */ Data, EncodedImageFormat, Font, FontMgr, Image, MaskFilter, Paint, RRect,
    Rect,
};
use std::{collections::HashMap, env::var, error::Error, fs, sync::Arc};
use tokio::sync::RwLock;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;
use twilight_model::{gateway::payload::incoming::MessageCreate, http::attachment::Attachment};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
type State = Arc<StateRef>;

const MUSIC_NOTE: &[u8] = include_bytes!("/home/aiving/music_note.svg");

struct StateRef {
    http: Arc<HttpClient>,
    genshin_data: Arc<RwLock<Option<GenshinData>>>,
    safebooru_data: Arc<RwLock<SafebooruData>>,
}

#[derive(Indexable, Debug, Deserialize)]
struct SafebooruData {
    posts: Vec<Post>,
    thumbnails: Vec<Vec<u8>>,
}

#[derive(Indexable, Debug, Deserialize)]
struct GenshinData {
    avatar: Vec<u8>,
    user_info: FullUserInfo,
    game_record: GameRecordCard,
    index: GenshinIndex,
    characters: Characters,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct BlackRelation {
    pub is_blacking: bool,
    pub is_blacked: bool,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct CollectionInfo {
    pub num: i32,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct CreatorInfo {
    pub can_collect: bool,
    pub can_top: bool,
    pub card_type: String,
    pub card_url: String,
    pub show_beta: bool,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct PaladinInfo {
    pub path: String,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct RequestingInfo {
    pub last_requesting_time: String,
    pub can_requesting: bool,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct LevelExp {
    pub level: u8,
    pub exp: u64,
    pub game_id: u8,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct Level {
    pub bg_color: String,
    pub bg_image: String,
    pub exp: u64,
    pub level: u8,
    pub level_desc: String,
}

fn parse_u32<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<u32, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrU32 {
        String(String),
        Number(u32),
    }

    match StringOrU32::deserialize(deserializer)? {
        StringOrU32::String(s) => s.parse().map_err(serde::de::Error::custom),
        StringOrU32::Number(i) => Ok(i),
    }
}

#[derive(Indexable, Debug, Deserialize)]
pub struct UserInfo {
    #[serde(deserialize_with = "parse_u32")]
    pub avatar: u32,
    pub avatar_url: String,
    pub bg_url: String,
    pub gender: u8,
    pub introduce: String,
    pub nickname: String,
    pub pc_bg_url: String,
    pub pendant: String,
    pub level: Level,
    pub level_exps: Vec<LevelExp>,
    #[serde(deserialize_with = "parse_u32")]
    pub uid: u32,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct FullUserInfo {
    pub black_relation: BlackRelation,
    pub collection_info: CollectionInfo,
    pub creator_info: CreatorInfo,
    pub paladin_info: PaladinInfo,
    pub requesting_info: RequestingInfo,
    pub user_info: UserInfo,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct KeyValue {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: u8,
    pub value: String,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct Switch {
    pub is_public: bool,
    pub switch_id: u8,
    pub switch_name: String,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct GameRecordCard {
    pub background_color: String,
    pub background_image: String,
    pub background_image_v2: String,
    pub data: Vec<KeyValue>,
    pub data_switches: Vec<Switch>,
    pub game_id: u8,
    pub game_name: String,
    #[serde(deserialize_with = "parse_u32")]
    pub game_role_id: u32,
    pub has_role: bool,
    pub is_public: bool,
    pub level: u8,
    pub logo: String,
    pub nickname: String,
    pub region: String,
    pub region_name: String,
    pub url: String,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct List<T> {
    pub list: Vec<T>,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct Response<T> {
    pub data: Option<T>,
    pub message: String,
    pub retcode: i32,
}

#[derive(Indexable, Debug)]
struct ResponseError {
    pub retcode: i32,
}

unsafe impl Send for ResponseError {}
unsafe impl Sync for ResponseError {}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { retcode } = self;

        write!(f, "E[{retcode}]")
    }
}

impl Error for ResponseError {}

impl<T> Response<T> {
    fn into_result(self) -> Result<T> {
        let Self {
            data,
            message: _,
            retcode,
        } = self;

        data.map_or_else(
            || Err(Box::new(ResponseError { retcode }) as Box<(dyn Error + Send + Sync + 'static)>),
            Ok,
        )
    }
}

#[derive(Indexable, Debug, Deserialize)]
pub struct Avatar {
    pub id: u32,
    pub name: String,
    pub image: String,
    pub card_image: String,
    pub level: u8,
    pub rarity: u8,
    pub fetter: u8,
    pub is_chosen: bool,
    pub element: String,
    pub actived_constellation_num: u8,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct Home {
    pub comfort_level_icon: String,
    pub comfort_level_name: String,
    pub comfort_num: u32,
    pub name: String,
    pub icon: String,
    pub item_num: u32,
    pub level: u8,
    pub visit_num: u32,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct Role {
    #[serde(rename = "AvatarUrl")]
    pub avatar_url: String,
    pub game_head_icon: String,
    pub nickname: String,
    pub region: String,
    pub level: u8,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct RoleCombat {
    pub is_unlock: bool,
    pub max_round_id: u32,
    pub has_data: bool,
    pub has_detail_data: bool,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct GenshinStats {
    pub active_day_number: u32,
    pub achievement_number: u32,
    pub anemoculus_number: u32,
    pub geoculus_number: u32,
    pub avatar_number: u32,
    pub way_point_number: u32,
    pub domain_number: u32,
    pub spiral_abyss: String,
    pub precious_chest_number: u32,
    pub luxurious_chest_number: u32,
    pub exquisite_chest_number: u32,
    pub common_chest_number: u32,
    pub electroculus_number: u32,
    pub magic_chest_number: u32,
    pub dendroculus_number: u32,
    pub hydroculus_number: u32,
    pub role_combat: RoleCombat,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct WorldExploration {
    pub level: u8,
    pub exploration_percentage: u32,
    pub icon: String,
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    // offerings: [],
    pub id: u8,
    pub parent_id: u8,
    pub map_url: String,
    pub strategy_url: String,
    pub background_image: String,
    pub inner_icon: String,
    pub cover: String,
    // area_exploration_list: [],
    // boss_list: [],
    pub is_hot: bool,
    pub index_active: bool,
    pub detail_active: bool,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct GenshinIndex {
    pub avatars: Vec<Avatar>,
    pub homes: Vec<Home>,
    pub role: Role,
    pub stats: GenshinStats,
    pub world_explorations: Vec<WorldExploration>,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct Property {
    pub filter_name: String,
    pub icon: String,
    pub name: String,
    pub property_type: u32,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct RelicOptions {
    pub sand_main_property_list: Vec<u8>,
    pub goblet_main_property_list: Vec<u8>,
    pub circlet_main_property_list: Vec<u8>,
    pub sub_property_list: Vec<u8>,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct PropertyValue {
    pub property_type: u32,
    pub base: String,
    pub add: String,
    #[serde(rename = "final")]
    pub output: String,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct Constellation {
    pub id: u32,
    pub name: String,
    pub effect: String,
    pub icon: String,
    pub is_actived: bool,
    pub pos: u32,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct SkillAffix {
    pub name: String,
    pub value: String,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct Skill {
    pub skill_id: u32,
    pub skill_type: u32,
    pub skill_affix_list: Vec<SkillAffix>,
    pub name: String,
    pub level: u32,
    pub desc: String,
    pub icon: String,
    pub is_unlock: bool,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct Weapon {
    pub id: u32,
    pub name: String,
    pub icon: String,
    #[serde(rename = "type")]
    pub kind: u32,
    pub rarity: u32,
    pub level: u32,
    pub promote_level: u32,
    #[serde(rename = "type_name")]
    pub kind_name: String,
    pub desc: String,
    pub affix_level: u32,
    pub main_property: Option<PropertyValue>,
    pub sub_property: Option<PropertyValue>,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct Costume {
    pub id: u32,
    pub name: String,
    pub icon: String,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct BaseWeapon {
    pub id: u32,
    pub icon: String,
    #[serde(rename = "type")]
    pub kind: u32,
    pub rarity: u32,
    pub level: u32,
    pub affix_level: u32,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct BaseCharacter {
    pub id: u32,
    pub icon: String,
    pub name: String,
    pub element: String,
    pub fetter: u32,
    pub level: u32,
    pub rarity: u32,
    pub actived_constellation_num: u32,
    pub image: String,
    pub is_chosen: bool,
    pub side_icon: String,
    pub weapon_type: u32,
    pub weapon: BaseWeapon,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct Character {
    pub base: BaseCharacter,
    pub base_properties: Vec<PropertyValue>,
    pub constellations: Vec<Constellation>,
    pub costumes: Vec<Costume>,
    pub element_properties: Vec<PropertyValue>,
    pub extra_properties: Vec<PropertyValue>,
    pub skills: Vec<Skill>,
    pub weapon: Weapon,
    pub selected_properties: Vec<PropertyValue>,
}

#[derive(Indexable, Debug, Deserialize)]
pub struct Characters {
    pub avatar_wiki: HashMap<u32, String>,
    pub property_map: HashMap<u32, Property>,
    pub list: Vec<Character>,
    pub relic_property_options: RelicOptions,
    pub relic_wiki: HashMap<u32, String>,
    pub weapon_wiki: HashMap<u32, String>,
}

#[derive(Debug, Serialize)]
pub struct CharacterRequest<'a> {
    pub role_id: String,
    pub server: &'a str,
    pub character_ids: Vec<String>,
}

#[rusfit(base_url = "https://bbs-api-os.hoyolab.com")]
#[header("x-rpc-app_version" = "1.5.0")]
#[header("x-rpc-lang" = "ru-ru")]
#[header("x-rpc-language" = "ru-ru")]
trait HoYoLab {
    #[route("GET", "/community/user/wapi/getUserFullInfo")]
    fn user_info(gid: u8) -> Response<FullUserInfo>;

    #[route("GET", "/game_record/card/wapi/getGameRecordCard")]
    fn game_record_card(uid: u32) -> Response<List<GameRecordCard>>;

    #[route("GET", "/game_record/genshin/api/index")]
    fn genshin_index(server: &str, role_id: u32) -> Response<GenshinIndex>;

    #[route("POST", "/game_record/genshin/api/character/detail")]
    fn genshin_character_detail(#[body] body: CharacterRequest<'_>) -> Response<Characters>;
}

#[derive(Indexable, Debug, Deserialize)]
struct Post {
    change: u32,
    #[serde(deserialize_with = "parse_u32")]
    directory: u32,
    hash: String,
    height: u32,
    width: u32,
    id: u32,
    image: String,
    owner: String,
    parent_id: u32,
    rating: String,
    sample: bool,
    sample_height: u32,
    sample_width: u32,
    score: Option<u32>,
    tags: String,
}

#[rusfit(base_url = "https://safebooru.org")]
trait Safebooru {
    #[route("GET", "/index.php?page=dapi&s=post&q=index&json=1&limit=40")]
    fn posts(tags: String) -> Vec<Post>;
}

#[tokio::main]
async fn main() -> Result<()> {
    let (mut shard, state) = {
        let token = var("DISCORD_TOKEN").unwrap();
        let intents = Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES | Intents::MESSAGE_CONTENT;
        let shard = Shard::new(ShardId::ONE, token.clone(), intents);

        let http = Arc::new(HttpClient::new(token));

        (
            shard,
            Arc::new(StateRef {
                http,
                genshin_data: Arc::new(RwLock::new(None)),
                safebooru_data: Arc::new(RwLock::new(SafebooruData {
                    posts: Vec::new(),
                    thumbnails: Vec::new(),
                })),
            }),
        )
    };

    // Since we only care about messages, make the cache only process messages.
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    // Startup the event loop to process each event in the event stream as they
    // come in.

    loop {
        let event = match shard.next_event().await {
            Ok(event) => event,
            Err(source) => {
                println!("error receiving event");

                if source.is_fatal() {
                    break;
                }

                continue;
            }
        };

        // Update the cache.
        cache.update(&event);

        // Spawn a new task to handle the event
        match event {
            Event::Ready(bot) => println!("{} is ready!", bot.user.name),
            Event::MessageCreate(message) => {
                tokio::spawn(handle_message(message, state.clone()));
            }
            _ => {}
        }
    }

    Ok(())
}

fn get_image(
    bg: Argb,
    image: Argb,
    title_color: Argb,
    artist_color: Argb,
    metadata: Metadata,
    position: i64,
) -> Option<Vec<u8>> {
    let (width, height) = (480.0, 226.0);
    let mut surface = surfaces::raster_n32_premul((width as i32, height as i32))
        .expect("can't create skia surface");
    let canvas = surface.canvas();

    let mut background = Paint::default();

    background.set_color(bg.as_color());

    canvas.draw_round_rect(
        Rect::from_xywh(0.0, 0.0, width, height),
        24.0,
        24.0,
        &background,
    );

    let image_size = 150.0;
    let image_shape = RRect::new_rect_xy(
        Rect::from_xywh(16.0, 50.0, image_size, image_size),
        16.0,
        16.0,
    );

    let mut image_paint = Paint::default();

    image_paint.set_color(image.as_color());

    canvas.draw_rrect(image_shape, &image_paint);

    if let Some(image) = metadata
        .art_url
        .as_ref()
        .and_then(|url| url.strip_prefix("file://"))
        .and_then(Data::from_filename)
        .and_then(Image::from_encoded)
    {
        let mut overlay_paint = Paint::default();

        overlay_paint.set_color(title_color.as_4color(50));

        overlay_paint.set_mask_filter(MaskFilter::blur(BlurStyle::Normal, 4.0, None));

        canvas.draw_rrect(image_shape, &overlay_paint);
        canvas.save();
        canvas.clip_rrect(image_shape, None, Some(true));
        canvas.draw_image(image, (16.0, 50.0), None);
        canvas.restore();
    }

    let fonts_manager = FontMgr::new();
    let title_font =
        fs::read("/usr/share/fonts/adobe-source-code-pro/SourceCodePro-Bold.otf").unwrap();
    let artist_font =
        fs::read("/usr/share/fonts/adobe-source-code-pro/SourceCodePro-Regular.otf").unwrap();
    let title_typeface = fonts_manager.new_from_data(&title_font, None).expect("AA");
    let artist_typeface = fonts_manager.new_from_data(&artist_font, None).expect("AA");
    let title_font = Font::from_typeface(title_typeface, 24.0);
    let artist_font = Font::from_typeface(artist_typeface, 18.0);

    let title = metadata.title.expect("there is no title in metadata");
    let artists = metadata.artists.expect("there is no artists in metadata");
    let _album = metadata.album.expect("there is no album in metadata");

    let (currently_listening_width, _) = artist_font.measure_str("Сейчас я слушаю...", None);

    let mut title_paint = Paint::default();

    title_paint.set_color(title_color.as_color());

    let mut artist_paint = Paint::default();

    artist_paint.set_color(artist_color.as_color());

    let note_svg = Dom::from_bytes(MUSIC_NOTE, fonts_manager).unwrap();

    canvas.draw_round_rect(
        Rect::from_xywh(16.0, 14.0, currently_listening_width + 36.0, 24.0),
        12.0,
        12.0,
        &image_paint,
    );

    canvas.save_layer(&SaveLayerRec::default());
    canvas.translate((22.0, 18.0));
    canvas.scale((
        18.0 / note_svg.inner().fContainerSize.fWidth,
        18.0 / note_svg.inner().fContainerSize.fHeight,
    ));

    note_svg.render(canvas);

    artist_paint.set_blend_mode(BlendMode::SrcIn);

    canvas.draw_paint(&artist_paint);
    canvas.restore();

    artist_paint.set_blend_mode(BlendMode::SrcOver);

    canvas.draw_str(
        "Сейчас я слушаю...",
        (44.0, 32.0),
        &artist_font,
        &title_paint,
    );
    canvas.draw_str(title, (32.0 + image_size, 74.0), &title_font, &title_paint);
    canvas.draw_str(
        if artists.len() > 1 {
            format!(
                "{} (feat. {})",
                artists.first().unwrap(),
                artists[1..].join(", ")
            )
        } else {
            artists.join(", ")
        },
        (32.0 + image_size, 100.0),
        &artist_font,
        &artist_paint,
    );

    let length = metadata.length.expect("there is no length in metadata");

    let position_width = (width - 32.0) * (position as f32 / length as f32);
    let left_width = (width - 32.0) - position_width;

    canvas.draw_round_rect(
        Rect::from_xywh(16.0, height - 18.0, position_width, 8.0),
        4.0,
        4.0,
        &title_paint,
    );
    canvas.draw_round_rect(
        Rect::from_xywh(20.0 + position_width, height - 18.0, left_width, 8.0),
        4.0,
        4.0,
        &image_paint,
    );

    let image = surface.image_snapshot();

    image
        .encode(None, EncodedImageFormat::PNG, 100)
        .map(|data| data.to_vec())
}

impl HoYoLab {
    async fn fetch_data(&self) -> Result<GenshinData> {
        let user_info = self.user_info(2).await?.into_result()?;

        println!("info: OK");

        let game_record = self
            .game_record_card(user_info.user_info.uid)
            .await?
            .into_result()?
            .list
            .remove(0);

        println!("record: OK");

        let index = self
            .genshin_index(&game_record.region, game_record.game_role_id)
            .await?
            .into_result()?;

        println!("index: OK");

        let request = CharacterRequest {
            role_id: game_record.game_role_id.to_string(),
            server: &game_record.region,
            character_ids: index
                .avatars
                .iter()
                .map(|avatar| avatar.id.to_string())
                .collect(),
        };

        let characters = self
            .genshin_character_detail(request)
            .await
            .expect("failed to fetch characters")
            .into_result()?;

        println!("characters: OK");

        let avatar = fetch_image(&user_info.user_info.avatar_url).await?;

        Ok(GenshinData {
            avatar,
            user_info,
            game_record,
            index,
            characters,
        })
    }
}

async fn fetch_image<T: AsRef<str> + Send>(url: T) -> Result<Vec<u8>> {
    Ok(reqwest::get(url.as_ref()).await?.bytes().await?.into())
}

trait ArgbExt {
    fn as_color(&self) -> Color;
    fn as_4color(&self, alpha: u8) -> Color;
    // fn as_4fcolor(&self) -> Color4f;
}

impl ArgbExt for Argb {
    fn as_color(&self) -> Color {
        Color::from_rgb(self.red, self.green, self.blue)
    }

    fn as_4color(&self, alpha: u8) -> Color {
        Color::from_argb(alpha, self.red, self.green, self.blue)
    }

    // fn as_4fcolor(&self) -> Color4f {
    //     Color4f::new(
    //         self.red as f32 / 255.0,
    //         self.green as f32 / 255.0,
    //         self.blue as f32 / 255.0,
    //         1.0,
    //     )
    // }
}

fn render_genshin(node: &Node) -> Option<Vec<u8>> {
    let mut context = RenderContext::new(1920, 1080)?;

    context.render(node);

    context.encode()
}

async fn handle_message(message: Box<MessageCreate>, state: State) -> Result<()> {
    if message.content.starts_with(":get-playing") {
        if let Some(player) = Player::find_active().await {
            let metadata = player
                .get_metadata()
                .await
                .expect("failed to get current track metadata");

            let position = player
                .get_position()
                .await
                .expect("failed to get current track position");

            let source_color = metadata
                .art_url
                .as_ref()
                .and_then(|url| url.strip_prefix("file://"))
                .as_ref()
                .and_then(|path| ImageReader::open(path).ok())
                .map_or(Argb::from_u32(0xFF32A852), |image| {
                    ImageReader::extract_color(&image)
                });

            let theme = ThemeBuilder::with_source(source_color)
                .variant(Variant::Content)
                .build()
                .schemes
                .dark;

            let image = get_image(
                theme.surface_container,
                theme.surface_bright,
                theme.primary,
                theme.secondary,
                metadata,
                position,
            );

            match image {
                Some(data) => {
                    state
                        .http
                        .create_message(message.channel_id)
                        .attachments(&[Attachment::from_bytes("image.png".to_owned(), data, 1)])?
                        .await?;
                }
                None => {
                    state
                        .http
                        .create_message(message.channel_id)
                        .content("failed to encode surface as PNG")?
                        .await?;
                }
            }
        }
    } else if let Some(layout) = message.content.strip_prefix(":genshin ") {
        if state.genshin_data.read().await.is_none() {
            let client = HoYoLab::new();

            match client.fetch_data().await {
                Ok(data) => {
                    state.genshin_data.write().await.replace(data);
                }
                Err(error) => {
                    state
                        .http
                        .create_message(message.channel_id)
                        .content(&format!("failed to fetch data: {error}"))?
                        .await?;
                }
            }
        }

        if let Some(data) = state.genshin_data.read().await.as_ref() {
            let theme = ThemeBuilder::with_source(Argb::from_u32(0xFF32A852))
                .build()
                .schemes
                .dark;

            let mut parser = Parser::new(Lexer::parse(layout));

            match parse_node(&mut parser, &theme.into_iter().collect(), Some(data)) {
                Ok(node) => match render_genshin(&node) {
                    Some(data) => {
                        state
                            .http
                            .create_message(message.channel_id)
                            .attachments(&[Attachment::from_bytes(
                                "image.png".to_owned(),
                                data,
                                1,
                            )])?
                            .await?;
                    }
                    None => {
                        state
                            .http
                            .create_message(message.channel_id)
                            .content("failed to encode surface as PNG")?
                            .await?;
                    }
                },
                Err(error) => {
                    state
                        .http
                        .create_message(message.channel_id)
                        .content(&format!("failed to parse: {error}"))?
                        .await?;
                }
            }
        }
    } else if let Some(tags) = message.content.strip_prefix(":safebooru ") {
        if state.safebooru_data.read().await.posts.is_empty() {
            let client = Safebooru::new();

            match client.posts(tags.into()).await {
                Ok(data) => {
                    let mut safebooru = state.safebooru_data.write().await;

                    safebooru.posts.extend(data.into_iter());

                    let mut thumbnails = Vec::new();

                    for post in &safebooru.posts {
                        if let Ok(image) = fetch_image(&format!(
                            "https://safebooru.org/thumbnails/{}/thumbnail_{}.jpg",
                            post.directory, &post.image[..post.image.len() - 4]
                        ))
                        .await
                        {
                            thumbnails.push(image);
                        }
                    }

                    safebooru.thumbnails = thumbnails;
                }
                Err(error) => {
                    state
                        .http
                        .create_message(message.channel_id)
                        .content(&format!("failed to fetch data: {error}"))?
                        .await?;
                }
            }
        }

        let data = &state.safebooru_data.read().await as &SafebooruData;

        let theme = ThemeBuilder::with_source(Argb::from_u32(0xFF32A852))
            .build()
            .schemes
            .dark;

        let node = Node::masonry(175.0)
            .size_p(100.0)
            .padding(8.0)
            .spacing(8.0)
            .background(theme.surface_container_high)
            .children(
                data.posts
                    .iter()
                    .zip(data.thumbnails.clone())
                    .map(|(post, data)| {
                        Node::image(data, &post.image)
                            .dynamic_height(vec![Operation::Mul(
                                Length::Width,
                                Length::Px(post.height as f32 / post.width as f32),
                            )])
                            .corner_radius(12.0)
                            .build()
                    })
                    .collect(),
            )
            .build();

        match render_genshin(&node) {
            Some(data) => {
                state
                    .http
                    .create_message(message.channel_id)
                    .attachments(&[Attachment::from_bytes("image.png".to_owned(), data, 1)])?
                    .await?;
            }
            None => {
                state
                    .http
                    .create_message(message.channel_id)
                    .content("failed to encode surface as PNG")?
                    .await?;
            }
        }
    }

    Ok(())
}
