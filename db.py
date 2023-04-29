import telebot
import sqlite3
import threading
from consts import BOT_ID

con = None
cur = None
lock = None

def save_message(message: telebot.types.Message):
    with lock:
        reply_id = message.reply_to_message.id if message.reply_to_message else 'None'
        is_assistant = int(message.from_user.id == BOT_ID)
        cur.execute('INSERT INTO messages(chat_id, msg_id, reply_id, assistant, sender, message) VALUES (?, ?, ?, ?, ?, ?)', (message.chat.id, message.id, reply_id, is_assistant, message.from_user.full_name, message.text))
        con.commit()

def init():
    global con
    global cur
    global lock
    con = sqlite3.connect('file:cachedb?mode=memory&cache=shared', check_same_thread=False)
    cur = con.cursor()
    cur.execute('CREATE TABLE IF NOT EXISTS messages(id INTEGER PRIMARY KEY, chat_id TEXT, msg_id TEXT, reply_id TEXT, assistant INTEGER, sender TEXT, message TEXT)')
    cur.execute('CREATE TABLE IF NOT EXISTS captions(id INTEGER PRIMARY KEY, chat_id TEXT, msg_id TEXT, caption TEXT)')
    con.commit()
    lock = threading.Lock()