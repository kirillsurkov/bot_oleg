import telebot
import telebot.formatting
from enum import Enum, auto
from bot import bot
from consts import WHITELIST, BOT_ID
from api_nekto import NektoApi

apis = []
chats_by_message = {}
apis_by_message = {}
messages_by_api = {}
mitm_2to1 = {}
mitm_1to2 = {}
relay_messages = []

class ChatMessageType(Enum):
    System = auto()
    Companion = auto()
    You = auto()
    Companion1 = auto()
    Companion1You = auto()
    Companion2 = auto()
    Companion2You = auto()

class ChatMessage:
    def __init__(self, text, msg_type):
        self.text = text
        self.type = msg_type

    def to_string(self):
        sender = ''
        if self.type == ChatMessageType.System:
            return telebot.formatting.escape_html(self.text)
        elif self.type == ChatMessageType.Companion:
            sender = 'Собеседник'
        elif self.type == ChatMessageType.Companion1:
            sender = 'Собеседник 1'
        elif self.type == ChatMessageType.Companion1You:
            sender = 'Собеседник 1 (Вы)'
        elif self.type == ChatMessageType.Companion2:
            sender = 'Собеседник 2'
        elif self.type == ChatMessageType.Companion2You:
            sender = 'Собеседник 2 (Вы)'
        elif self.type == ChatMessageType.You:
            sender = 'Вы'
        return f'<b>{sender}</b>: {telebot.formatting.escape_html(self.text)}'

def nekto_filter(message: telebot.types.Message):
    return message.reply_to_message and message.reply_to_message.from_user.id == BOT_ID and (message.reply_to_message.chat.id, message.reply_to_message.id) in chats_by_message

def update_message(message: telebot.types.Message):
    try:
        bot.edit_message_text('\n'.join([m.to_string() for m in chats_by_message[(message.chat.id, message.id)][-150:]]), chat_id=message.chat.id, message_id=message.id, parse_mode='HTML')
    except:
        pass

def on_state(api: NektoApi):
    if id(api) not in messages_by_api:
        return
    message = messages_by_api[id(api)]
    if api.state == NektoApi.State.Disconnected:
        api.connect()
    elif api.state == NektoApi.State.Connecting:
        chats_by_message[(message.chat.id, message.id)] += [ChatMessage('Подключение...', ChatMessageType.System)]
    elif api.state == NektoApi.State.Connected:
        chats_by_message[(message.chat.id, message.id)] += [ChatMessage('Подключено', ChatMessageType.System)]
    elif api.state == NektoApi.State.Logined:
        chats_by_message[(message.chat.id, message.id)] += [ChatMessage('Вход выполнен', ChatMessageType.System)]
        api.search()
    elif api.state == NektoApi.State.CaptchaRequired:
        chats_by_message[(message.chat.id, message.id)] += [ChatMessage('Капча...', ChatMessageType.System)]
    elif api.state == NektoApi.State.CaptchaSolved:
        chats_by_message[(message.chat.id, message.id)] += [ChatMessage('Капча решена', ChatMessageType.System)]
        api.search()
    elif api.state == NektoApi.State.Searching:
        chats_by_message[(message.chat.id, message.id)] += [ChatMessage('Поиск...', ChatMessageType.System)]
    elif api.state == NektoApi.State.InChat:
        chats_by_message[(message.chat.id, message.id)] = [ChatMessage(f'{api.companion_id}: Собеседник найден!', ChatMessageType.System)]
    update_message(message)

def get_available_api():
    global apis
    available = [api for api in apis if id(api) not in messages_by_api]
    if len(available) == 0:
        print('Creating new API')
        api = NektoApi()
        apis += [api]
        return api
    else:
        return available[0]

@bot.message_handler(commands=['nekto'])
def on_nekto(message: telebot.types.Message):
    if message.chat.id not in WHITELIST:
        return

    def on_end(api: NektoApi):
        message = messages_by_api[id(api)]
        chats_by_message[(message.chat.id, message.id)] += [ChatMessage('Собеседник покинул чат.', ChatMessageType.System)]
        update_message(message)
        del messages_by_api[id(api)]
        del apis_by_message[(message.chat.id, message.id)]
        del chats_by_message[(message.chat.id, message.id)]

    def on_message(api: NektoApi, sender: int, text: str):
        message = messages_by_api[id(api)]
        msg_type = ChatMessageType.You if sender == api.id else ChatMessageType.Companion
        chats_by_message[(message.chat.id, message.id)] += [ChatMessage(text, msg_type)]
        update_message(message)

    message = bot.reply_to(message, 'Загрузка...')
    api = get_available_api()
    api.on_state = on_state
    api.on_end = on_end
    api.on_message = on_message
    chats_by_message[(message.chat.id, message.id)] = []
    apis_by_message[(message.chat.id, message.id)] = [api]
    messages_by_api[id(api)] = message
    on_state(api)

@bot.message_handler(commands=['nekto2'])
def on_nekto2(message: telebot.types.Message):
    if message.chat.id not in WHITELIST:
        return

    def get_mitm_data(api: NektoApi):
        message = messages_by_api[id(api)]
        message = mitm_2to1[(message.chat.id, message.id)]
        api_id = None
        apis = []
        for i, m in enumerate(mitm_1to2[(message.chat.id, message.id)]):
            apis += [apis_by_message[(m.chat.id, m.id)][0]]
            if apis_by_message[(m.chat.id, m.id)][0].companion_id == api.companion_id:
                api_id = i
        return (message, api_id, apis[api_id - 1])

    def on_end(api: NektoApi):
        message, _, api_companion = get_mitm_data(api)
        api_companion.end()
        chats_by_message[(message.chat.id, message.id)] += [ChatMessage('Собеседник покинул чат.', ChatMessageType.System)]
        update_message(message)

        message = messages_by_api[id(api)]
        bot.delete_message(chat_id=message.chat.id, message_id=message.id)
        del messages_by_api[id(api)]

    def on_message(api: NektoApi, sender: int, text: str):
        if sender != api.companion_id:
            return
        msg_types = ()
        if sender == api.companion_id:
            msg_types = (ChatMessageType.Companion1, ChatMessageType.Companion2)
        else:
            msg_types = (ChatMessageType.Companion1You, ChatMessageType.Companion2You)
        message, api_id, api_companion = get_mitm_data(api)
        api_companion.send(text)
        msg_type = msg_types[api_id]
        chats_by_message[(message.chat.id, message.id)] += [ChatMessage(text, msg_type)]
        update_message(message)

    apis = []
    messages = []
    for _ in range(2):
        api = get_available_api()
        apis += [api]
        api.on_state = on_state
        api.on_end = on_end
        api.on_message = on_message
        msg = bot.reply_to(message, 'Загрузка...')
        messages += [msg]
        chats_by_message[(msg.chat.id, msg.id)] = []
        apis_by_message[(msg.chat.id, msg.id)] = [api]
        messages_by_api[id(api)] = msg
    msg = bot.reply_to(message, 'Загрузка...')
    mitm_1to2[(msg.chat.id, msg.id)] = messages
    for m in messages:
        mitm_2to1[(m.chat.id, m.id)] = msg
    for api in apis:
        on_state(api)
    chats_by_message[(msg.chat.id, msg.id)] = []
    apis_by_message[(msg.chat.id, msg.id)] = apis
    messages_by_api[(id(api) for api in apis)] = msg

@bot.message_handler(commands=['end'])
def on_end(message: telebot.types.Message):
    if message.chat.id not in WHITELIST:
        return
    if message.reply_to_message and (message.reply_to_message.chat.id, message.reply_to_message.id) in apis_by_message:
        for api in apis_by_message[(message.reply_to_message.chat.id, message.reply_to_message.id)]:
            api.end()

@bot.message_handler(func=nekto_filter)
def on_reply(message: telebot.types.Message):
    if message.chat.id not in WHITELIST:
        return
    if message.reply_to_message and (message.reply_to_message.chat.id, message.reply_to_message.id) in apis_by_message:
        if (message.chat.id, message.id) not in mitm_1to2 and (message.chat.id, message.id) not in mitm_2to1:
            for api in apis_by_message[(message.reply_to_message.chat.id, message.reply_to_message.id)]:
                api.send(message.text)
                bot.delete_message(chat_id=message.chat.id, message_id=message.id)