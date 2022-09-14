use hyper::body::Buf;
use hyper::client::{Client, HttpConnector};
use hyper::{Body, Method, Request, StatusCode, Uri};
use hyper_tls::HttpsConnector;

use std::str::FromStr;

use crate::models::{DiscordApiCompatible, Message, MessageContext, Webhook};

pub type WebhookResult<Type> = std::result::Result<Type, Box<dyn std::error::Error + Send + Sync>>;

/// A Client that sends webhooks for discord.
pub struct WebhookClient {
    client: Client<HttpsConnector<HttpConnector>>,
    url: String,
}

impl WebhookClient {
    pub fn new(url: &str) -> Self {
        let https_connector = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https_connector);
        Self {
            client,
            url: url.to_owned(),
        }
    }

    /// Example
    /// ```ignore
    /// let client = WebhookClient::new("URL");
    /// client.send(|message| message
    ///     .content("content")
    ///     .username("username")).await?;
    /// ```
    pub async fn send<Func>(&self, function: Func) -> WebhookResult<bool>
    where
        Func: Fn(&mut Message) -> &mut Message,
    {
        let mut message = Message::new();
        function(&mut message);
        let mut message_context = MessageContext::new();
        match message.check_compatibility(&mut message_context) {
            Ok(_) => (),
            Err(error_message) => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    error_message,
                )));
            }
        };
        let result = self.send_message(&message).await?;

        Ok(result)
    }

    pub async fn send_message(&self, message: &Message) -> WebhookResult<bool> {
        let body = serde_json::to_string(message)?;
        let request = Request::builder()
            .method(Method::POST)
            .uri(&self.url)
            .header("content-type", "application/json")
            .body(Body::from(body))?;
        let response = self.client.request(request).await?;

        // https://discord.com/developers/docs/resources/webhook#execute-webhook
        // execute webhook returns either NO_CONTENT or a message
        if response.status() == StatusCode::NO_CONTENT {
            Ok(true)
        } else {
            let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
            let err_msg = match String::from_utf8(body_bytes.to_vec()) {
                Ok(msg) => msg,
                Err(err) => {
                    "Error reading Discord API error message:".to_string() + &err.to_string()
                }
            };

            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                err_msg,
            )))
        }
    }

    pub async fn get_information(&self) -> WebhookResult<Webhook> {
        let response = self.client.get(Uri::from_str(&self.url)?).await?;
        let body = hyper::body::aggregate(response).await?;
        let webhook = serde_json::from_reader(body.reader())?;

        Ok(webhook)
    }
}

#[cfg(test)]
mod tests {
    use crate::client::WebhookClient;
    use crate::models::{
        ActionRow, DiscordApiCompatible, Message, MessageContext, NonLinkButtonStyle, SelectMenu,
        SelectOption,
    };

    async fn assert_client_error<BuildFunc, MessagePred>(
        message_build: BuildFunc,
        msg_pred: MessagePred,
    ) -> ()
    where
        BuildFunc: Fn(&mut Message) -> &mut Message,
        MessagePred: Fn(&str) -> bool,
    {
        let client = WebhookClient::new("https://discord.com");
        let result = client.send(message_build).await;
        match result {
            Err(err) => {
                assert!(
                    msg_pred(&err.to_string()),
                    "Unexpected error message {}",
                    err.to_string()
                )
            }
            Ok(_) => assert!(false, "Error is expected"),
        };
    }

    fn contains_all_predicate(needles: Vec<&str>) -> Box<dyn Fn(&str) -> bool> {
        let owned_needles: Vec<String> = needles.iter().map(|n| n.to_string()).collect();
        Box::new(move |haystack| {
            let lower_haystack = haystack.to_lowercase();
            owned_needles
                .iter()
                .all(|needle| lower_haystack.contains(needle))
        })
    }

    fn assert_valid_message<BuildFunc>(func: BuildFunc)
    where
        BuildFunc: Fn(&mut Message) -> &mut Message,
    {
        let mut message = Message::new();
        func(&mut message);
        if let Err(unexpected) = message.check_compatibility(&mut MessageContext::new()) {
            assert!(false, "Unexpected validation error {}", unexpected);
        }
    }

    #[tokio::test]
    async fn empty_action_row_prohibited() {
        assert_client_error(
            |message| message.action_row(|row| row),
            contains_all_predicate(vec!["action row", "empty"]),
        )
        .await;
    }

    #[tokio::test]
    async fn send_message_custom_id_reuse_prohibited() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    row.regular_button(|button| {
                        button.custom_id("0").style(NonLinkButtonStyle::Primary)
                    })
                    .regular_button(|button| {
                        button.custom_id("0").style(NonLinkButtonStyle::Primary)
                    })
                })
            },
            contains_all_predicate(vec!["twice"]),
        )
        .await;
    }

    #[tokio::test]
    async fn send_message_custom_id_reuse_prohibited_accross_action_rows() {
        assert_client_error(
            |message| {
                message
                    .action_row(|row| {
                        row.regular_button(|button| {
                            button.custom_id("0").style(NonLinkButtonStyle::Primary)
                        })
                    })
                    .action_row(|row| {
                        row.regular_button(|button| {
                            button.custom_id("0").style(NonLinkButtonStyle::Primary)
                        })
                    })
            },
            contains_all_predicate(vec!["twice"]),
        )
        .await;
    }

    #[tokio::test]
    async fn send_message_button_style_required() {
        assert_client_error(
            |message| message.action_row(|row| row.regular_button(|button| button.custom_id("0"))),
            contains_all_predicate(vec!["style"]),
        )
        .await;
    }

    #[tokio::test]
    async fn send_message_url_required() {
        assert_client_error(
            |message| message.action_row(|row| row.link_button(|button| button.label("test"))),
            contains_all_predicate(vec!["url"]),
        )
        .await;
    }

    #[tokio::test]
    async fn send_message_max_action_rows_enforced() {
        assert_client_error(
            |message| {
                for _ in 0..(Message::action_row_count_interval().max_allowed + 1) {
                    message.action_row(|row| row);
                }
                message
            },
            contains_all_predicate(vec!["interval", "row"]),
        )
        .await;
    }

    #[tokio::test]
    async fn send_message_max_label_len_enforced() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    row.regular_button(|btn| {
                        btn.style(NonLinkButtonStyle::Primary)
                            .custom_id("a")
                            .label(&"l".repeat(Message::label_len_interval().max_allowed + 1))
                    })
                })
            },
            contains_all_predicate(vec!["interval", "label"]),
        )
        .await;
    }

    #[tokio::test]
    async fn send_message_custom_id_required() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    row.regular_button(|btn| btn.style(NonLinkButtonStyle::Primary))
                })
            },
            contains_all_predicate(vec!["custom id"]),
        )
        .await;
    }

    #[tokio::test]
    async fn send_message_max_custom_id_len_enforced() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    row.regular_button(|btn| {
                        btn.style(NonLinkButtonStyle::Primary).custom_id(
                            &"a".repeat(Message::custom_id_len_interval().max_allowed + 1),
                        )
                    })
                })
            },
            contains_all_predicate(vec!["interval", "custom id"]),
        )
        .await;
    }

    #[tokio::test]
    async fn max_button_count_enforced() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    for i in 0..(ActionRow::button_count_interval().max_allowed + 1) {
                        row.regular_button(|btn| {
                            btn.style(NonLinkButtonStyle::Primary)
                                .custom_id(&(i.to_string()))
                        });
                    }
                    row
                })
            },
            contains_all_predicate(vec!["interval", "button"]),
        )
        .await;
    }

    #[test]
    fn max_button_count_enforced_only_per_action_row() {
        assert_valid_message(|message| {
            for i in 0..Message::action_row_count_interval().max_allowed {
                message.action_row(|row| {
                    for j in 0..(ActionRow::button_count_interval().max_allowed) {
                        row.regular_button(|btn| {
                            btn.style(NonLinkButtonStyle::Primary)
                                .custom_id(&(i.to_string() + &j.to_string()))
                        });
                    }
                    row
                });
            }
            message
        });
    }

    #[tokio::test]
    async fn option_maximum_count_enforced() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    row.select_menu(|menu| {
                        for i in 0..SelectMenu::option_count_interval().max_allowed + 1 {
                            menu.option(|opt| opt.label(&i.to_string()).value(&i.to_string()));
                        }
                        menu.custom_id("test")
                    });
                    row
                })
            },
            contains_all_predicate(vec!["interval", "option", "count"]),
        )
        .await;
    }

    fn init_menu_options_and_skip_n(menu: &mut SelectMenu, skip_count: usize) -> &mut SelectMenu {
        for i in 0..(SelectMenu::option_count_interval().min_allowed - skip_count) {
            menu.option(|opt| opt.label(&i.to_string()).value(&i.to_string()));
        }
        menu
    }

    fn init_menu_options(menu: &mut SelectMenu) -> &mut SelectMenu {
        init_menu_options_and_skip_n(menu, 0)
    }

    #[tokio::test]
    async fn select_menu_custom_id_len_enforced() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    row.select_menu(|menu| {
                        init_menu_options(menu).custom_id(
                            &"t".repeat(Message::custom_id_len_interval().max_allowed + 1),
                        )
                    });
                    row
                })
            },
            contains_all_predicate(vec!["custom id", "interval"]),
        )
        .await;
    }

    #[tokio::test]
    async fn select_menu_placeholder_len_enforced() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    row.select_menu(|menu| {
                        init_menu_options(menu).custom_id("test").placeholder(
                            &"t".repeat(SelectMenu::placeholder_len_interval().max_allowed + 1),
                        )
                    });
                    row
                })
            },
            contains_all_predicate(vec!["placeholder", "interval"]),
        )
        .await;
    }

    #[tokio::test]
    async fn select_menu_min_values_maximum_enforced() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    row.select_menu(|menu| {
                        init_menu_options(menu)
                            .custom_id("test")
                            .min_values(SelectMenu::min_values_interval().max_allowed + 1)
                    });
                    row
                })
            },
            contains_all_predicate(vec!["min values", "interval"]),
        )
        .await;
    }

    #[tokio::test]
    async fn select_menu_max_values_maximum_enforced() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    row.select_menu(|menu| {
                        init_menu_options(menu)
                            .custom_id("test")
                            .max_values(SelectMenu::max_values_interval().max_allowed + 1)
                    });
                    row
                })
            },
            contains_all_predicate(vec!["max values", "interval"]),
        )
        .await;
    }

    #[tokio::test]
    async fn max_select_menu_count_enforced() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    for i in 0..(ActionRow::select_menu_count_interval().max_allowed + 1) {
                        row.select_menu(|menu| {
                            menu.custom_id(&i.to_string())
                                .option(|opt| opt.label("test").value("test"))
                        });
                    }
                    row
                })
            },
            contains_all_predicate(vec!["interval", "select menu"]),
        )
        .await;
    }

    #[tokio::test]
    async fn select_option_label_len_enforced() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    row.select_menu(|menu| {
                        init_menu_options_and_skip_n(menu, 1)
                            .custom_id("test")
                            .option(|opt| {
                                opt.label(
                                    &"l".repeat(SelectOption::label_len_interval().max_allowed + 1),
                                )
                                .value("test")
                            })
                    });
                    row
                })
            },
            contains_all_predicate(vec!["label", "interval"]),
        )
        .await;
    }

    #[tokio::test]
    async fn select_option_value_len_enforced() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    row.select_menu(|menu| {
                        init_menu_options_and_skip_n(menu, 1)
                            .custom_id("test")
                            .option(|opt| {
                                opt.value(
                                    &"v".repeat(SelectOption::value_len_interval().max_allowed + 1),
                                )
                                .label("test")
                            })
                    });
                    row
                })
            },
            contains_all_predicate(vec!["value", "interval"]),
        )
        .await;
    }

    #[tokio::test]
    async fn select_option_description_len_enforced() {
        assert_client_error(
            |message| {
                message.action_row(|row| {
                    row.select_menu(|menu| {
                        init_menu_options_and_skip_n(menu, 1)
                            .custom_id("test")
                            .option(|opt| {
                                opt.description(&"d".repeat(
                                    SelectOption::description_len_interval().max_allowed + 1,
                                ))
                                .value("test")
                                .label("test")
                            })
                    });
                    row
                })
            },
            contains_all_predicate(vec!["description", "interval"]),
        )
        .await;
    }

    #[test]
    fn message_valid_basic() {
        assert_valid_message(|message| {
            message
                .content("@test")
                .username("test")
                .avatar_url("test")
                .embed(|embed| {
                    embed
                        .title("test")
                        .description("test")
                        .footer("test", Some(String::from("test")))
                        .image("test")
                        .thumbnail("test")
                        .author(
                            "test",
                            Some(String::from("test")),
                            Some(String::from("test")),
                        )
                        .field("test", "test", false)
                })
        });
    }

    fn test_is_send<T>(t: T)
    where
        T: Send,
    {
        drop(t);
    }

    #[test]
    fn message_is_send() {
        let message = Message::new();
        // this should not compile if Message is not Send
        test_is_send(message);
    }
}
