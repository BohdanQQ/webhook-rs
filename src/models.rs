use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashSet;

type Snowflake = String;

#[derive(Deserialize, Debug)]
pub struct Webhook {
    pub id: Snowflake,
    #[serde(rename = "type")]
    pub webhook_type: i8,
    pub guild_id: Snowflake,
    pub channel_id: Snowflake,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub token: String,
    pub application_id: Option<Snowflake>,
}

#[derive(Debug)]
pub(crate) struct MessageContext {
    custom_ids: HashSet<String>,
    button_count_in_action_row: usize,
    select_menu_count_in_action_row: usize,
}

impl MessageContext {
    /// Tries to register a custom id.
    ///
    /// # Watch out!
    ///
    /// Use `register_button` and `register_select_menu` for Buttons and Select Menus respectively!
    ///
    /// # Arguments
    ///
    /// * `id`: the custom id to be registered
    ///
    ///
    /// # Return value
    /// Returns true if the custom id is unique.
    ///
    /// Returns false if the supplied custom id is duplicate of an already registered custom id.
    fn register_custom_id(&mut self, id: &str) -> Result<(), String> {
        if !self.custom_ids.insert(id.to_string()) {
            return Err(format!("Attempt to use the same custom ID ({}) twice!", id));
        }
        Ok(())
    }

    pub fn new() -> MessageContext {
        MessageContext {
            custom_ids: HashSet::new(),
            button_count_in_action_row: 0,
            select_menu_count_in_action_row: 0,
        }
    }

    //TODO - documentation
    pub fn register_button(&mut self, id: &str) -> Result<(), String> {
        self.register_custom_id(id)?;

        if self.button_count_in_action_row >= ActionRow::max_button_count() {
            return Err(format!(
                "Button count for action row exceeded maximum ({})",
                ActionRow::max_button_count()
            ));
        }

        self.button_count_in_action_row += 1;

        if self.select_menu_count_in_action_row > 0 {
            return Err(
                "An Action Row containing buttons cannot also contain a select menu".to_string(),
            );
        }

        Ok(())
    }

    pub fn register_select_menu(&mut self, id: &str) -> Result<(), String> {
        self.register_custom_id(id)?;

        if self.select_menu_count_in_action_row >= ActionRow::max_select_menu_count() {
            return Err(format!(
                "Select menu count for action row exceeded maximum ({})",
                ActionRow::max_select_menu_count()
            ));
        }

        self.select_menu_count_in_action_row += 1;

        if self.button_count_in_action_row > 0 {
            return Err(
                "An Action Row containing a select menu cannot also contain buttons".to_string(),
            );
        }

        Ok(())
    }

    // should be called once per action row
    pub fn register_action_row(&mut self) {
        self.button_count_in_action_row = 0;
        self.button_count_in_action_row = 0;
    }
}

#[derive(Serialize, Debug)]
pub struct Message {
    pub content: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub tts: bool,
    pub embeds: Vec<Embed>,
    pub allow_mentions: Option<AllowedMentions>,
    #[serde(rename = "components")]
    pub action_rows: Vec<ActionRow>,
}

impl Message {
    pub fn new() -> Self {
        Self {
            content: None,
            username: None,
            avatar_url: None,
            tts: false,
            embeds: vec![],
            allow_mentions: None,
            action_rows: vec![],
        }
    }

    pub fn content(&mut self, content: &str) -> &mut Self {
        self.content = Some(content.to_owned());
        self
    }

    pub fn username(&mut self, username: &str) -> &mut Self {
        self.username = Some(username.to_owned());
        self
    }

    pub fn avatar_url(&mut self, avatar_url: &str) -> &mut Self {
        self.avatar_url = Some(avatar_url.to_owned());
        self
    }

    pub fn tts(&mut self, tts: bool) -> &mut Self {
        self.tts = tts;
        self
    }

    pub fn embed<Func>(&mut self, func: Func) -> &mut Self
    where
        Func: Fn(&mut Embed) -> &mut Embed,
    {
        let mut embed = Embed::new();
        func(&mut embed);
        self.embeds.push(embed);

        self
    }

    pub fn action_row<Func>(&mut self, func: Func) -> &mut Self
    where
        Func: Fn(&mut ActionRow) -> &mut ActionRow,
    {
        let mut row = ActionRow::new();
        func(&mut row);
        self.action_rows.push(row);

        self
    }

    pub fn max_action_row_count() -> usize {
        5
    }

    pub fn label_max_len() -> usize {
        80
    }

    pub fn custom_id_max_len() -> usize {
        100
    }

    pub fn allow_mentions(
        &mut self,
        parse: Option<Vec<AllowedMention>>,
        roles: Option<Vec<Snowflake>>,
        users: Option<Vec<Snowflake>>,
        replied_user: bool,
    ) -> &mut Self {
        self.allow_mentions = Some(AllowedMentions::new(parse, roles, users, replied_user));
        self
    }
}

#[derive(Serialize, Debug)]
pub struct Embed {
    pub title: Option<String>,
    #[serde(rename = "type")]
    embed_type: String,
    pub description: Option<String>,
    pub url: Option<String>,
    // ISO8601,
    pub timestamp: Option<String>,
    pub color: Option<String>,
    pub footer: Option<EmbedFooter>,
    pub image: Option<EmbedImage>,
    pub video: Option<EmbedVideo>,
    pub thumbnail: Option<EmbedThumbnail>,
    pub provider: Option<EmbedProvider>,
    pub author: Option<EmbedAuthor>,
    pub fields: Vec<EmbedField>,
}

impl Embed {
    pub fn new() -> Self {
        Self {
            title: None,
            embed_type: String::from("rich"),
            description: None,
            url: None,
            timestamp: None,
            color: None,
            footer: None,
            image: None,
            video: None,
            thumbnail: None,
            provider: None,
            author: None,
            fields: vec![],
        }
    }

    pub fn title(&mut self, title: &str) -> &mut Self {
        self.title = Some(title.to_owned());
        self
    }

    pub fn description(&mut self, description: &str) -> &mut Self {
        self.description = Some(description.to_owned());
        self
    }

    pub fn url(&mut self, url: &str) -> &mut Self {
        self.url = Some(url.to_owned());
        self
    }

    pub fn timestamp(&mut self, timestamp: &str) -> &mut Self {
        self.timestamp = Some(timestamp.to_owned());
        self
    }

    pub fn color(&mut self, color: &str) -> &mut Self {
        self.color = Some(color.to_owned());
        self
    }

    pub fn footer(&mut self, text: &str, icon_url: Option<String>) -> &mut Self {
        self.footer = Some(EmbedFooter::new(text, icon_url));
        self
    }

    pub fn image(&mut self, url: &str) -> &mut Self {
        self.image = Some(EmbedImage::new(url));
        self
    }

    pub fn video(&mut self, url: &str) -> &mut Self {
        self.video = Some(EmbedVideo::new(url));
        self
    }

    pub fn thumbnail(&mut self, url: &str) -> &mut Self {
        self.thumbnail = Some(EmbedThumbnail::new(url));
        self
    }

    pub fn provider(&mut self, name: &str, url: &str) -> &mut Self {
        self.provider = Some(EmbedProvider::new(name, url));
        self
    }

    pub fn author(
        &mut self,
        name: &str,
        url: Option<String>,
        icon_url: Option<String>,
    ) -> &mut Self {
        self.author = Some(EmbedAuthor::new(name, url, icon_url));
        self
    }

    pub fn field(&mut self, name: &str, value: &str, inline: bool) -> &mut Self {
        if self.fields.len() == 25 {
            panic!("You can't have more than 25 fields in an embed!")
        }

        self.fields.push(EmbedField::new(name, value, inline));
        self
    }
}

#[derive(Serialize, Debug)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

impl EmbedField {
    pub fn new(name: &str, value: &str, inline: bool) -> Self {
        Self {
            name: name.to_owned(),
            value: value.to_owned(),
            inline,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct EmbedFooter {
    pub text: String,
    pub icon_url: Option<String>,
}

impl EmbedFooter {
    pub fn new(text: &str, icon_url: Option<String>) -> Self {
        Self {
            text: text.to_owned(),
            icon_url,
        }
    }
}

pub type EmbedImage = EmbedUrlSource;
pub type EmbedThumbnail = EmbedUrlSource;
pub type EmbedVideo = EmbedUrlSource;

#[derive(Serialize, Debug)]
pub struct EmbedUrlSource {
    pub url: String,
}

impl EmbedUrlSource {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct EmbedProvider {
    pub name: String,
    pub url: String,
}

impl EmbedProvider {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_owned(),
            url: url.to_owned(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct EmbedAuthor {
    pub name: String,
    pub url: Option<String>,
    pub icon_url: Option<String>,
}

impl EmbedAuthor {
    pub fn new(name: &str, url: Option<String>, icon_url: Option<String>) -> Self {
        Self {
            name: name.to_owned(),
            url,
            icon_url,
        }
    }
}

pub enum AllowedMention {
    RoleMention,
    UserMention,
    EveryoneMention,
}

fn resolve_allowed_mention_name(allowed_mention: AllowedMention) -> String {
    match allowed_mention {
        AllowedMention::RoleMention => "roles".to_string(),
        AllowedMention::UserMention => "users".to_string(),
        AllowedMention::EveryoneMention => "everyone".to_string(),
    }
}

#[derive(Serialize, Debug)]
pub struct AllowedMentions {
    pub parse: Option<Vec<String>>,
    pub roles: Option<Vec<Snowflake>>,
    pub users: Option<Vec<Snowflake>>,
    pub replied_user: bool,
}

impl AllowedMentions {
    pub fn new(
        parse: Option<Vec<AllowedMention>>,
        roles: Option<Vec<Snowflake>>,
        users: Option<Vec<Snowflake>>,
        replied_user: bool,
    ) -> Self {
        let mut parse_strings: Vec<String> = vec![];
        if parse.is_some() {
            parse
                .unwrap()
                .into_iter()
                .for_each(|x| parse_strings.push(resolve_allowed_mention_name(x)))
        }

        Self {
            parse: Some(parse_strings),
            roles,
            users,
            replied_user,
        }
    }
}

// ready to be extended with other components
// non-composite here specifically means *not an action row*
#[derive(Debug)]
enum NonCompositeComponent {
    Button(Button),
    SelectMenu(SelectMenu),
}

impl Serialize for NonCompositeComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            NonCompositeComponent::Button(button) => button.serialize(serializer),
            NonCompositeComponent::SelectMenu(menu) => menu.serialize(serializer),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ActionRow {
    #[serde(rename = "type")]
    pub component_type: u8,
    components: Vec<NonCompositeComponent>,
}

impl ActionRow {
    fn new() -> ActionRow {
        ActionRow {
            component_type: 1,
            components: vec![],
        }
    }
    //TODO: (consider) change return type (limit the interface) once Button or SelectMenu is added? (limited to only one of those)
    pub fn link_button<Func>(&mut self, button_mutator: Func) -> &mut Self
    where
        Func: Fn(&mut LinkButton) -> &mut LinkButton,
    {
        let mut button = LinkButton::new();
        button_mutator(&mut button);
        self.components.push(NonCompositeComponent::Button(
            button.to_serializable_button(),
        ));
        self
    }

    pub fn regular_button<Func>(&mut self, button_mutator: Func) -> &mut Self
    where
        Func: Fn(&mut RegularButton) -> &mut RegularButton,
    {
        let mut button = RegularButton::new();
        button_mutator(&mut button);
        self.components.push(NonCompositeComponent::Button(
            button.to_serializable_button(),
        ));
        self
    }

    pub fn select_menu<Func>(&mut self, menu_mutator: Func) -> &mut Self
    where
        Func: Fn(&mut SelectMenu) -> &mut SelectMenu,
    {
        let mut menu = SelectMenu::new(None, None, None, None, None);
        menu_mutator(&mut menu);
        self.components
            .push(NonCompositeComponent::SelectMenu(menu));
        self
    }

    pub fn max_button_count() -> usize {
        5
    }

    pub fn max_select_menu_count() -> usize {
        1
    }
}

#[derive(Debug, Clone)]
pub enum NonLinkButtonStyle {
    Primary,
    Secondary,
    Success,
    Danger,
}

impl NonLinkButtonStyle {
    fn get_button_style(&self) -> ButtonStyles {
        match *self {
            NonLinkButtonStyle::Primary => ButtonStyles::Primary,
            NonLinkButtonStyle::Secondary => ButtonStyles::Secondary,
            NonLinkButtonStyle::Success => ButtonStyles::Success,
            NonLinkButtonStyle::Danger => ButtonStyles::Danger,
        }
    }
}

// since link button has an explicit way of creation via the action row
// this enum is kept hidden from the user ans the NonLinkButtonStyle is created to avoid
// user confusion
#[derive(Debug)]
enum ButtonStyles {
    Primary,
    Secondary,
    Success,
    Danger,
    Link,
}

impl ButtonStyles {
    /// value for serialization purposes
    fn value(&self) -> i32 {
        match *self {
            ButtonStyles::Primary => 1,
            ButtonStyles::Secondary => 2,
            ButtonStyles::Success => 3,
            ButtonStyles::Danger => 4,
            ButtonStyles::Link => 5,
        }
    }
}

impl Serialize for ButtonStyles {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.value())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct PartialEmoji {
    pub id: Snowflake,
    pub name: String,
    pub animated: Option<bool>,
}

/// the button struct intended for serialized
#[derive(Serialize, Debug)]
struct Button {
    #[serde(rename = "type")]
    pub component_type: i8,
    pub style: Option<ButtonStyles>,
    pub label: Option<String>,
    pub emoji: Option<PartialEmoji>,
    pub custom_id: Option<String>,
    pub url: Option<String>,
    pub disabled: Option<bool>,
}

impl Button {
    fn new(
        style: Option<ButtonStyles>,
        label: Option<String>,
        emoji: Option<PartialEmoji>,
        url: Option<String>,
        custom_id: Option<String>,
        disabled: Option<bool>,
    ) -> Self {
        Self {
            component_type: 2,
            style,
            label,
            emoji,
            url,
            custom_id,
            disabled,
        }
    }
}

/// Data holder for shared fields of link and regular buttons
#[derive(Debug)]
struct ButtonCommonBase {
    pub label: Option<String>,
    pub emoji: Option<PartialEmoji>,
    pub disabled: Option<bool>,
}

impl ButtonCommonBase {
    fn new(label: Option<String>, emoji: Option<PartialEmoji>, disabled: Option<bool>) -> Self {
        ButtonCommonBase {
            label,
            emoji,
            disabled,
        }
    }
    fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    fn emoji(&mut self, emoji_id: Snowflake, name: &str, animated: bool) -> &mut Self {
        self.emoji = Some(PartialEmoji {
            id: emoji_id,
            name: name.to_string(),
            animated: Some(animated),
        });
        self
    }

    fn disabled(&mut self, disabled: bool) -> &mut Self {
        self.disabled = Some(disabled);
        self
    }
}

/// a macro which takes an identifier (`base`) of the ButtonCommonBase (relative to `self`)
/// and generates setter functions that delegate their inputs to the `self.base`
macro_rules! button_base_delegation {
    ($base:ident) => {
        pub fn emoji(&mut self, emoji_id: &str, name: &str, animated: bool) -> &mut Self {
            self.$base.emoji(emoji_id.to_string(), name, animated);
            self
        }

        pub fn disabled(&mut self, disabled: bool) -> &mut Self {
            self.$base.disabled(disabled);
            self
        }

        pub fn label(&mut self, label: &str) -> &mut Self {
            self.$base.label(label);
            self
        }
    };
}

#[derive(Debug)]
pub struct LinkButton {
    button_base: ButtonCommonBase,
    url: Option<String>,
}

impl LinkButton {
    fn new() -> Self {
        LinkButton {
            button_base: ButtonCommonBase::new(None, None, None),
            url: None,
        }
    }

    pub fn url(&mut self, url: &str) -> &mut Self {
        self.url = Some(url.to_string());
        self
    }

    button_base_delegation!(button_base);
}

pub struct RegularButton {
    button_base: ButtonCommonBase,
    custom_id: Option<String>,
    style: Option<NonLinkButtonStyle>,
}

impl RegularButton {
    fn new() -> Self {
        RegularButton {
            button_base: ButtonCommonBase::new(None, None, None),
            custom_id: None,
            style: None,
        }
    }

    pub fn custom_id(&mut self, custom_id: &str) -> &mut Self {
        self.custom_id = Some(custom_id.to_string());
        self
    }

    pub fn style(&mut self, style: NonLinkButtonStyle) -> &mut Self {
        self.style = Some(style);
        self
    }

    button_base_delegation!(button_base);
}

trait ToSerializableButton {
    fn to_serializable_button(&self) -> Button;
}

impl ToSerializableButton for LinkButton {
    fn to_serializable_button(&self) -> Button {
        Button::new(
            Some(ButtonStyles::Link),
            self.button_base.label.clone(),
            self.button_base.emoji.clone(),
            self.url.clone(),
            None,
            self.button_base.disabled,
        )
    }
}

impl ToSerializableButton for RegularButton {
    fn to_serializable_button(&self) -> Button {
        Button::new(
            self.style.clone().map(|s| s.get_button_style()),
            self.button_base.label.clone(),
            self.button_base.emoji.clone(),
            None,
            self.custom_id.clone(),
            self.button_base.disabled,
        )
    }
}

macro_rules! string_option_setter {
    ($base:ident) => {
        pub fn $base(&mut self, $base: &str) -> &mut Self {
            self.$base = Some($base.to_string());
            self
        }
    };
}

macro_rules! simple_option_setter {
    ($base:ident, $option_inner_t:ty) => {
        pub fn $base(&mut self, $base: $option_inner_t) -> &mut Self {
            self.$base = Some($base);
            self
        }
    };
}

#[derive(Serialize, Debug)]
pub struct SelectMenu {
    #[serde(rename = "type")]
    pub component_type: i8,
    pub custom_id: Option<String>,
    pub options: Vec<SelectOption>,
    pub placeholder: Option<String>,
    pub min_values: Option<u8>,
    pub max_values: Option<u8>,
    pub disabled: Option<bool>,
}

impl SelectMenu {
    fn new(
        custom_id: Option<String>,
        placeholder: Option<String>,
        min_values: Option<u8>,
        max_values: Option<u8>,
        disabled: Option<bool>,
    ) -> Self {
        Self {
            component_type: 3,
            custom_id,
            options: vec![],
            placeholder,
            min_values,
            max_values,
            disabled,
        }
    }

    pub fn option<Func>(&mut self, option_mutator: Func) -> &mut Self
    where
        Func: Fn(&mut SelectOption) -> &mut SelectOption,
    {
        let mut option = SelectOption::new(None, None, None, None, None);
        option_mutator(&mut option);
        self.options.push(option);
        self
    }

    string_option_setter!(custom_id);
    string_option_setter!(placeholder);
    simple_option_setter!(min_values, u8);
    simple_option_setter!(max_values, u8);
    simple_option_setter!(disabled, bool);

    pub fn max_option_count() -> usize {
        25
    }
    pub fn placeholder_max_len() -> usize {
        150
    }
    pub fn minimum_of_min_values() -> u8 {
        0
    }
    pub fn maximum_of_min_values() -> u8 {
        25
    }
    pub fn maximum_of_max_values() -> u8 {
        25
    }
}

#[derive(Serialize, Debug)]
pub struct SelectOption {
    pub label: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
    pub emoji: Option<PartialEmoji>,
    pub default: Option<bool>,
}

impl SelectOption {
    fn new(
        label: Option<String>,
        value: Option<String>,
        description: Option<String>,
        emoji: Option<PartialEmoji>,
        default: Option<bool>,
    ) -> Self {
        Self {
            label,
            value,
            description,
            emoji,
            default,
        }
    }

    string_option_setter!(label);
    string_option_setter!(value);
    string_option_setter!(description);

    pub fn emoji(&mut self, emoji_id: &str, name: &str, animated: bool) -> &mut Self {
        self.emoji = Some(PartialEmoji {
            id: emoji_id.to_string(),
            name: name.to_string(),
            animated: Some(animated),
        });
        self
    }

    simple_option_setter!(default, bool);

    pub fn label_max_len() -> usize {
        100
    }
    pub fn value_max_len() -> usize {
        100
    }
    pub fn description_max_len() -> usize {
        100
    }
}

/// A trait for checking that an API message component is compatible with the official Discord API constraints
///
/// This trait should be implemented for any components for which the Discord API documentation states
/// limitations (maximum count, maximum length, uniqueness with respect to other components, restrictions
/// on children components, ...)
pub(crate) trait DiscordApiCompatible {
    fn check_compatibility(&self, context: &mut MessageContext) -> Result<(), String>;
}

impl DiscordApiCompatible for NonCompositeComponent {
    fn check_compatibility(&self, context: &mut MessageContext) -> Result<(), String> {
        match self {
            NonCompositeComponent::Button(b) => b.check_compatibility(context),
            NonCompositeComponent::SelectMenu(m) => m.check_compatibility(context),
        }
    }
}

fn bool_to_result<E>(b: bool, err: E) -> Result<(), E> {
    if b {
        Ok(())
    } else {
        Err(err)
    }
}

impl DiscordApiCompatible for Button {
    fn check_compatibility(&self, context: &mut MessageContext) -> Result<(), String> {
        if self.label.is_some() && self.label.as_ref().unwrap().len() > Message::label_max_len() {
            return Err(format!(
                "Label length exceeds {} characters",
                Message::label_max_len()
            ));
        }

        return match self.style {
            None => Err("Button style must be set!".to_string()),
            Some(ButtonStyles::Link) => {
                if self.url.is_none() {
                    Err("Url of a Link button must be set!".to_string())
                } else {
                    Ok(())
                }
            }
            // list all remaining in case a style with different requirements is added
            Some(ButtonStyles::Danger)
            | Some(ButtonStyles::Primary)
            | Some(ButtonStyles::Success)
            | Some(ButtonStyles::Secondary) => {
                return if let Some(id) = self.custom_id.as_ref() {
                    bool_to_result(
                        id.len() <= Message::custom_id_max_len(),
                        format!(
                            "Custom ID length exceeds {} characters",
                            Message::custom_id_max_len()
                        ),
                    )
                    .and(context.register_button(id))
                } else {
                    Err("Custom ID of a NonLink button must be set!".to_string())
                };
            }
        };
    }
}

impl DiscordApiCompatible for ActionRow {
    fn check_compatibility(&self, context: &mut MessageContext) -> Result<(), String> {
        context.register_action_row();
        self.components.iter().fold(Ok(()), |acc, component| {
            acc.and(component.check_compatibility(context))
        })
    }
}

impl DiscordApiCompatible for Message {
    fn check_compatibility(&self, context: &mut MessageContext) -> Result<(), String> {
        if self.action_rows.len() > Self::max_action_row_count() {
            return Err(format!(
                "Action row count exceeded {} (maximum)",
                Message::max_action_row_count()
            ));
        }

        self.action_rows
            .iter()
            .fold(Ok(()), |acc, row| acc.and(row.check_compatibility(context)))
    }
}

fn prepare_length_check<'b>(
    specs: Vec<(&Option<String>, usize, &'b str)>,
) -> Vec<(String, usize, &'b str)> {
    let mut result = vec![];
    for (opt, size, err_name) in specs {
        if let Some(val) = opt {
            result.push((val.clone(), size, err_name));
        }
    }
    result
}

fn limit_max_length(specs: Vec<(&Option<String>, usize, &str)>) -> Result<(), String> {
    let length_check_specification = prepare_length_check(specs);
    for (value, max_len, err_name) in length_check_specification {
        if value.len() > max_len {
            return Err(format!("{} exceeded maximum length {}", err_name, max_len));
        }
    }
    Ok(())
}

impl DiscordApiCompatible for SelectOption {
    fn check_compatibility(&self, _context: &mut MessageContext) -> Result<(), String> {
        if self.label.is_none() {
            return Err("Label of a menu option must be set!".to_string());
        } else if self.value.is_none() {
            return Err("Value of a menu option must be set!".to_string());
        }

        limit_max_length(vec![
            (&self.label, SelectOption::label_max_len(), "Label"),
            (&self.label, SelectOption::value_max_len(), "Value"),
            (
                &self.description,
                SelectOption::description_max_len(),
                "Description",
            ),
        ])
    }
}

impl DiscordApiCompatible for SelectMenu {
    fn check_compatibility(&self, context: &mut MessageContext) -> Result<(), String> {
        if let Some(id) = self.custom_id.as_ref() {
            context.register_select_menu(id)?
        } else {
            return Err("Custom ID of a Select menu must be set!".to_string());
        }

        if self.options.len() > SelectMenu::max_option_count() {
            return Err(format!(
                "Option count exceeded maximum {}",
                SelectMenu::max_option_count()
            ));
        } else if self.min_values.map_or(false, |val| {
            val > SelectMenu::maximum_of_min_values() || val < SelectMenu::minimum_of_min_values()
        }) {
            return Err(format!(
                "Min values does not fall into the [{}, {}] interval",
                SelectMenu::minimum_of_min_values(),
                SelectMenu::maximum_of_min_values()
            ));
        } else if self
            .max_values
            .map_or(false, |val| val > SelectMenu::maximum_of_max_values())
        {
            return Err(format!(
                "Max values exceeded maximum {}",
                SelectMenu::maximum_of_max_values()
            ));
        }

        limit_max_length(vec![
            (&self.custom_id, Message::custom_id_max_len(), "Custom ID"),
            (
                &self.placeholder,
                SelectMenu::placeholder_max_len(),
                "Placeholder",
            ),
        ])
        .and(
            self.options
                .iter()
                .fold(Ok(()), |acc, val| acc.and(val.check_compatibility(context))),
        )
    }
}
