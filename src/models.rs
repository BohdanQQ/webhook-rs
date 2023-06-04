use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashSet;
use std::fmt::Display;
type Snowflake = String;

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

macro_rules! interval_getter {
    ($name:ident, $option_inner_t:ty, $lower_bound:expr, $upper_bound:expr) => {
        pub const fn $name() -> Interval<$option_inner_t> {
            Interval::from_min_max($lower_bound, $upper_bound)
        }
    };
}

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

    string_option_setter!(content);
    string_option_setter!(username);
    string_option_setter!(avatar_url);

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

    interval_getter!(action_row_count_interval, usize, 0, 5);
    interval_getter!(label_len_interval, usize, 0, 80);
    interval_getter!(custom_id_len_interval, usize, 1, 100);

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

pub struct Interval<T> {
    pub max_allowed: T,
    pub min_allowed: T,
}

impl<T: Ord> Interval<T> {
    pub const fn from_min_max(min_allowed: T, max_allowed: T) -> Self {
        Interval {
            min_allowed,
            max_allowed,
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        self.min_allowed <= *value && self.max_allowed >= *value
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

    string_option_setter!(title);
    string_option_setter!(description);
    string_option_setter!(url);
    string_option_setter!(timestamp);
    string_option_setter!(color);

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

    interval_getter!(button_count_interval, usize, 0, 5);
    interval_getter!(select_menu_count_interval, usize, 0, 1);
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

impl Serialize for ButtonStyles {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let to_serialize = match *self {
            ButtonStyles::Primary => 1,
            ButtonStyles::Secondary => 2,
            ButtonStyles::Success => 3,
            ButtonStyles::Danger => 4,
            ButtonStyles::Link => 5,
        };
        serializer.serialize_i32(to_serialize)
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

    string_option_setter!(label);

    fn emoji(&mut self, emoji_id: Snowflake, name: &str, animated: bool) -> &mut Self {
        self.emoji = Some(PartialEmoji {
            id: emoji_id,
            name: name.to_string(),
            animated: Some(animated),
        });
        self
    }
    simple_option_setter!(disabled, bool);
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

    string_option_setter!(url);

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

    string_option_setter!(custom_id);
    simple_option_setter!(style, NonLinkButtonStyle);

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

    interval_getter!(option_count_interval, usize, 1, 25);
    interval_getter!(placeholder_len_interval, usize, 0, 150);
    interval_getter!(min_values_interval, u8, 0, 25);

    // the minimum is not actually stated, but at the time of implementing the API returns an error response when max_values == 0
    // additionally, max_values == 0 doesn't really make sense
    interval_getter!(max_values_interval, u8, 1, 25);
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
    interval_getter!(label_len_interval, usize, 1, 100);
    interval_getter!(value_len_interval, usize, 1, 100);
    interval_getter!(description_len_interval, usize, 0, 100);
}

#[derive(Debug)]
pub(crate) struct MessageContext {
    custom_ids: HashSet<String>,
    button_count_in_action_row: usize,
    select_menu_count_in_action_row: usize,
}

fn interval_check<T: Ord + Display>(
    interval: &Interval<T>,
    value_to_test: &T,
    field_name: &str,
) -> Result<(), String> {
    if !interval.contains(value_to_test) {
        return Err(format!(
            "{} ({}) not in the [{}, {}] interval",
            field_name, value_to_test, interval.min_allowed, interval.max_allowed
        ));
    }
    Ok(())
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
    /// Error variant contains an error message
    fn register_custom_id(&mut self, id: &str) -> Result<(), String> {
        interval_check(
            &Message::custom_id_len_interval(),
            &id.len(),
            "Custom ID length",
        )?;

        if !self.custom_ids.insert(id.to_string()) {
            return Err(format!("Attempt to use the same custom ID ({}) twice!", id));
        }
        Ok(())
    }

    pub(crate) fn new() -> MessageContext {
        MessageContext {
            custom_ids: HashSet::new(),
            button_count_in_action_row: 0,
            select_menu_count_in_action_row: 0,
        }
    }

    /// Tries to register a button using the button's custom id.
    ///
    /// # Return value
    /// Error variant contains an error message
    ///
    /// # Note
    /// Subsequent calls register other components semantically in the same action row.
    /// To register components in a new action row, use the `register_action_row` function before
    /// calling this function
    fn register_button(&mut self, id: &str) -> Result<(), String> {
        self.register_custom_id(id)?;
        self.button_count_in_action_row += 1;

        interval_check(
            &ActionRow::button_count_interval(),
            &self.button_count_in_action_row,
            "Button count",
        )?;

        if self.select_menu_count_in_action_row > 0 {
            return Err(
                "An Action Row containing buttons cannot also contain a select menu".to_string(),
            );
        }

        Ok(())
    }

    /// Tries to register a select menu using the button's custom id
    ///
    /// # Return value
    /// Error variant contains an error message
    ///
    /// # Note
    /// Subsequent calls register other components semantically in the same action row.
    /// To register components in a new action row, use the `register_action_row` function before
    /// calling this function
    fn register_select_menu(&mut self, id: &str) -> Result<(), String> {
        self.register_custom_id(id)?;
        self.select_menu_count_in_action_row += 1;

        interval_check(
            &ActionRow::select_menu_count_interval(),
            &self.select_menu_count_in_action_row,
            "Select menu count",
        )?;

        if self.button_count_in_action_row > 0 {
            return Err(
                "An Action Row containing a select menu cannot also contain buttons".to_string(),
            );
        }

        Ok(())
    }

    /// Switches the context to register components logically in a "new" action row.
    ///
    /// # Watch out!
    /// This function shall be called only once per one action row. (due to the lack of action row
    /// identification)
    fn register_action_row(&mut self) {
        self.button_count_in_action_row = 0;
        self.button_count_in_action_row = 0;
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

impl DiscordApiCompatible for Button {
    fn check_compatibility(&self, context: &mut MessageContext) -> Result<(), String> {
        if let Some(label) = &self.label {
            interval_check(&Message::label_len_interval(), &label.len(), "Label length")?;
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
                    context.register_button(id)
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
        if self.components.is_empty() {
            return Err("Empty action row detected!".to_string());
        }

        self.components.iter().fold(Ok(()), |acc, component| {
            acc.and(component.check_compatibility(context))
        })
    }
}

impl DiscordApiCompatible for Embed {
    fn check_compatibility(&self, context: &mut MessageContext) -> Result<(), String> {
        todo!()
    }
}

impl DiscordApiCompatible for Message {
    fn check_compatibility(&self, context: &mut MessageContext) -> Result<(), String> {
        interval_check(
            &Message::action_row_count_interval(),
            &self.action_rows.len(),
            "Action row count",
        )?;

        self.action_rows
            .iter()
            .fold(Ok(()), |acc, row| acc.and(row.check_compatibility(context))).and(self.embeds.iter()
            .fold(Ok(()), |acc, embed| acc.and(embed.check_compatibility(context))))
    }
}

impl DiscordApiCompatible for SelectOption {
    fn check_compatibility(&self, _context: &mut MessageContext) -> Result<(), String> {
        if let Some(label) = &self.label {
            interval_check(
                &SelectOption::label_len_interval(),
                &label.len(),
                "Label length",
            )?;
        } else {
            return Err("Label of a menu option must be set!".to_string());
        }

        if let Some(value) = &self.value {
            interval_check(
                &SelectOption::value_len_interval(),
                &value.len(),
                "Value length",
            )?;
        } else {
            return Err("Value of a menu option must be set!".to_string());
        }

        if let Some(desc) = &self.description {
            interval_check(
                &SelectOption::description_len_interval(),
                &desc.len(),
                "Description length",
            )?;
        }
        Ok(())
    }
}

impl DiscordApiCompatible for SelectMenu {
    fn check_compatibility(&self, context: &mut MessageContext) -> Result<(), String> {
        if let Some(id) = self.custom_id.as_ref() {
            context.register_select_menu(id)?
        } else {
            return Err("Custom ID of a Select menu must be set!".to_string());
        }

        interval_check(
            &SelectMenu::option_count_interval(),
            &self.options.len(),
            "Option count",
        )?;

        let mut min = 0;
        let mut max = 0;
        if let Some(min_values) = self.min_values {
            interval_check(
                &SelectMenu::min_values_interval(),
                &min_values,
                "Min values",
            )?;
            min = min_values;
        }
        if let Some(max_values) = self.max_values {
            interval_check(
                &SelectMenu::max_values_interval(),
                &max_values,
                "Max values",
            )?;
            max = max_values;
        }
        if self.min_values.is_some() && self.max_values.is_some() && min > max {
            return Err(format!(
                "Min values ({}) more than max values ({})",
                min, max
            ));
        }

        if let Some(placeholder) = &self.placeholder {
            interval_check(
                &SelectMenu::placeholder_len_interval(),
                &placeholder.len(),
                "Placeholder length",
            )?;
        }

        self.options
            .iter()
            .fold(Ok(()), |acc, val| acc.and(val.check_compatibility(context)))
    }
}
