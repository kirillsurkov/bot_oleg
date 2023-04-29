import telebot
import random
from bot import bot

@bot.message_handler(commands=['qrand'])
def on_help(message: telebot.types.Message):
    is_fire = random.randint(0, 10)
    print(is_fire)
    if is_fire == 0:
        bot.send_sticker(message.chat.id, 'CAACAgIAAxkBAAIDT2QaBiVLd2FiMJ4jgenuUAABm7u4ywACRisAAuTzyEjW7K7f9yGmJC8E', message.id)