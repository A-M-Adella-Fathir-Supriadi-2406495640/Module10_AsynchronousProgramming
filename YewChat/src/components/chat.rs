use serde::{Deserialize, Serialize};
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{services::event_bus::EventBus, services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let val = input.value();
                    if val.trim().is_empty() {
                        return false;
                    }
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(val),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);

        // Enter key sends message
        let onkeypress = ctx.link().batch_callback(|e: KeyboardEvent| {
            if e.key() == "Enter" {
                Some(Msg::SubmitMessage)
            } else {
                None
            }
        });

        let user_count = self.users.len();
        let msg_count = self.messages.len();

        html! {
            <div class="flex w-screen h-screen bg-gray-900 text-white">

                // ── Sidebar ──────────────────────────────────────────
                <div class="flex-none w-64 h-screen bg-gray-800 flex flex-col border-r border-gray-700">
                    // Sidebar header
                    <div class="px-4 py-4 border-b border-gray-700">
                        <div class="flex items-center gap-2">
                            <span class="text-2xl">{"🦀"}</span>
                            <span class="font-bold text-lg text-orange-400">{"RustChat"}</span>
                        </div>
                        <p class="text-gray-400 text-xs mt-1">
                            {format!("{} user{} online", user_count, if user_count == 1 { "" } else { "s" })}
                        </p>
                    </div>

                    // User list
                    <div class="flex-grow overflow-y-auto py-2">
                        <p class="text-gray-500 text-xs font-semibold uppercase tracking-widest px-4 py-2">
                            {"Online"}
                        </p>
                        {
                            self.users.clone().iter().map(|u| {
                                html!{
                                    <div class="flex items-center gap-3 px-4 py-2 hover:bg-gray-700 rounded-lg mx-2 transition-colors">
                                        <div class="relative flex-none">
                                            <img
                                                class="w-9 h-9 rounded-full bg-gray-600"
                                                src={u.avatar.clone()}
                                                alt="avatar"
                                            />
                                            // Green online dot
                                            <span class="absolute bottom-0 right-0 w-2.5 h-2.5 bg-green-400 rounded-full border-2 border-gray-800"></span>
                                        </div>
                                        <div class="min-w-0">
                                            <p class="text-sm font-medium text-gray-200 truncate">{u.name.clone()}</p>
                                            <p class="text-xs text-green-400">{"● online"}</p>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>

                    // Sidebar footer — GIF tip
                    <div class="px-4 py-3 border-t border-gray-700">
                        <p class="text-gray-500 text-xs">{"💡 Tip: send a .gif URL to share an animation!"}</p>
                    </div>
                </div>

                // ── Main chat area ────────────────────────────────────
                <div class="flex flex-col flex-grow h-screen overflow-hidden">

                    // Header
                    <div class="flex items-center justify-between px-6 py-4 bg-gray-800 border-b border-gray-700 flex-none">
                        <div class="flex items-center gap-2">
                            <span class="text-xl">{"💬"}</span>
                            <span class="font-semibold text-white">{"General"}</span>
                        </div>
                        <span class="text-xs text-gray-400">
                            {format!("{} message{}", msg_count, if msg_count == 1 { "" } else { "s" })}
                        </span>
                    </div>

                    // Messages
                    <div class="flex-grow overflow-y-auto px-6 py-4 space-y-4">
                        {
                            if self.messages.is_empty() {
                                html! {
                                    <div class="flex flex-col items-center justify-center h-full text-center text-gray-600">
                                        <div class="text-5xl mb-3">{"🦀"}</div>
                                        <p class="font-semibold">{"No messages yet"}</p>
                                        <p class="text-sm mt-1">{"Be the first to say something!"}</p>
                                    </div>
                                }
                            } else {
                                self.messages.iter().map(|m| {
                                    let user = self.users.iter().find(|u| u.name == m.from);
                                    let avatar = user.map(|u| u.avatar.clone()).unwrap_or_default();
                                    html!{
                                        <div class="flex items-end gap-3 max-w-2xl">
                                            <img
                                                class="w-8 h-8 rounded-full flex-none bg-gray-700"
                                                src={avatar}
                                                alt="avatar"
                                            />
                                            <div class="bg-gray-700 rounded-tl-2xl rounded-tr-2xl rounded-br-2xl px-4 py-3 max-w-lg">
                                                <p class="text-xs font-semibold text-orange-400 mb-1">{m.from.clone()}</p>
                                                <div class="text-sm text-gray-200">
                                                    {
                                                        if m.message.ends_with(".gif") {
                                                            html! { <img class="mt-2 rounded-lg max-w-xs" src={m.message.clone()}/> }
                                                        } else {
                                                            html! { {m.message.clone()} }
                                                        }
                                                    }
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        }
                    </div>

                    // Input bar
                    <div class="flex-none px-6 py-4 bg-gray-800 border-t border-gray-700">
                        <div class="flex items-center gap-3 bg-gray-700 rounded-2xl px-4 py-2">
                            <input
                                ref={self.chat_input.clone()}
                                type="text"
                                placeholder="Type a message… (Enter to send)"
                                class="flex-grow bg-transparent outline-none text-gray-200 placeholder-gray-500 text-sm"
                                name="message"
                                required=true
                                onkeypress={onkeypress}
                            />
                            <button
                                onclick={submit}
                                class="flex-none w-9 h-9 bg-orange-500 hover:bg-orange-400 rounded-xl flex items-center justify-center transition-colors"
                            >
                                <svg fill="white" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-4 h-4">
                                    <path d="M0 0h24v24H0z" fill="none"></path>
                                    <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                                </svg>
                            </button>
                        </div>
                        <p class="text-gray-600 text-xs mt-1 ml-2">{"Press Enter or click the button to send"}</p>
                    </div>
                </div>
            </div>
        }
    }
}
