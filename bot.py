import telebot
from consts import BOT_TOKEN

bot = None

def init():
    global bot
    bot = telebot.TeleBot(BOT_TOKEN)