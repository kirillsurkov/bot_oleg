import telebot
import assistant
from consts import BOT_ID, BOT_ALIAS, OPENAI_TOKEN, MEMORY_SIZE, OLEG_PROMPT
from bot import bot

import openai
openai.api_key = OPENAI_TOKEN

def oleg_filter(message: telebot.types.Message):
    text = message.text or message.caption
    if text and text.split(' ', 1)[0] == '/oleg':
        message.text = text.split('/oleg', 1)
        if len(message.text) > 1:
            message.text = message.text[1].strip()
        else:
            message.text = ''
        return True
    if message.reply_to_message and message.reply_to_message.from_user.id == BOT_ID:
        return True
    if message.entities:
        for e in message.entities:
            if e.type == "mention" and message.text[e.offset:e.offset+e.length] == BOT_ALIAS:
                return True
    return False

def get_gpt35(messages: list):
    def make_gpt35_message(msg_type: int, content: str, name: str = None):
        res = {'role': ['system', 'assistant', 'user'][msg_type]}
        if msg_type == assistant.MSG_USR:
            res['content'] = (f'{name}: ' if name else '') + content
        else:
            res['content'] = content
        return res

    init = [
        make_gpt35_message(assistant.MSG_USR, OLEG_PROMPT),
        make_gpt35_message(assistant.MSG_ASS, 'Хорошо.'),
    ]
    messages = init + [make_gpt35_message(assistant.MSG_ASS if m[1] else assistant.MSG_USR, m[3], name=m[2]) for m in messages]
    print(messages)
    completion = openai.ChatCompletion.create(model='gpt-3.5-turbo', messages=messages, temperature=1.0)
    return completion['choices'][0]['message']['content'].strip()

@bot.message_handler(func=oleg_filter, content_types=['text', 'photo'])
def on_mention(message: telebot.types.Message):
    assistant.do_reply(message, get_gpt35, MEMORY_SIZE)