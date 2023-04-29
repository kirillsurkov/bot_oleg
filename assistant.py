import telebot
import requests
import traceback
import db
from consts import WHITELIST, SD_URL
from utils import img2base64
from bot import bot

MSG_SYS = 0
MSG_ASS = 1
MSG_USR = 2

def get_answer(messages: list, callback):
    print("wait")
    answer = callback(messages)
    print(answer)

    answer = answer.split(' ')
    if len(answer) > 1 and len(answer[0]) > 0 and answer[0][-1] == ':':
        answer = answer[1:]
    elif len(answer) > 2 and len(answer[1]) > 0 and answer[1][-1] == ':':
        answer = answer[2:]
    answer = ' '.join(answer)

    return answer

def extract_caption(message: telebot.types.Message):
    if message and message.photo and len(message.photo) > 0:
        caption = None
        with db.lock:
            caption = db.con.execute('SELECT caption FROM captions WHERE chat_id=? and msg_id=?', (message.chat.id, message.id)).fetchone()
        if not caption:
            file_info = bot.get_file(message.photo[-1].file_id)
            response = requests.post(f'{SD_URL}/sdapi/v1/interrogate', json={
                'image': img2base64(bot.download_file(file_info.file_path)),
                'model': 'clip',
            }).json()
            print(response)
            if 'caption' in response:
                caption = response['caption']
                with db.lock:
                    db.con.execute('INSERT INTO captions(chat_id, msg_id, caption) VALUES(?, ?, ?)', (message.chat.id, message.id, caption))
                    db.con.commit()

def get_message_chain(chat_id: int, msg_id: int, limit: int):
    messages = []
    if limit > 0:
        for assistant, msg_id, sender, msg in db.con.execute('SELECT m2.assistant, m2.msg_id, m2.sender, m2.message FROM messages m1, messages m2 WHERE m1.chat_id=? and m1.msg_id=? and m1.reply_id=m2.msg_id', (chat_id, msg_id)):
            messages += [[msg_id, assistant, sender, msg]] + get_message_chain(chat_id, msg_id, limit-1)
    return messages

def collect_messages(chat_id: int, limit: int, from_id: int, buffer_size: int) -> list:
    messages = []
    with db.lock:
        max_msg_id = from_id if from_id is not None else 9223372036854775807
        chain_limit = buffer_size
        for (assistant, msg_id, sender, msg) in db.con.execute('SELECT assistant, msg_id, sender, message FROM messages WHERE chat_id=? and CAST(msg_id as integer) <= ? ORDER BY CAST(msg_id as integer) DESC LIMIT ?', (chat_id, max_msg_id, limit)):
            messages += [[msg_id, assistant, sender, msg]]
            messages += get_message_chain(chat_id, msg_id, chain_limit)
            chain_limit = 1
    #return list(reversed(messages))

    final_messages = []
    ids = []
    for message in messages:
        if message[0] in ids:
            continue
        ids += [message[0]]
        final_messages = [message] + final_messages
    return final_messages

def do_reply(message: telebot.types.Message, callback, buffer_size: int):
    if message.chat.id not in WHITELIST:
        return

    try:
        extract_caption(message)
        extract_caption(message.reply_to_message)

        messages = []
        if message.reply_to_message is not None:
            messages = collect_messages(message.chat.id, buffer_size, message.reply_to_message.id, buffer_size)
        if len(messages) == 0:
            messages = collect_messages(message.chat.id, buffer_size, None, buffer_size)
        if message.text and len(message.text) > 0:
            messages += [[message.id, False, message.from_user.full_name, message.text]]

        for m in messages:
            print(m)
            m[3] = m[3] or ''
            for (caption,) in db.con.execute('SELECT caption FROM captions WHERE chat_id=? and msg_id=?', (message.chat.id, m[0])):
                m[3] = 'Описание картинки: ' + caption + '\nЕсли хотите узнать, что на этой картинке - просто спросите.\n' + m[3]
                break

        reply = get_answer(messages[-buffer_size:], callback)
        try:
            answer = bot.reply_to(message, reply, parse_mode='MarkdownV2')
        except:
            answer = bot.reply_to(message, reply, parse_mode=None)
        db.save_message(message)
        db.save_message(answer)
    except Exception as e:
        traceback.print_exception(e)