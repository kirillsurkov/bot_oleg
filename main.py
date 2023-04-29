import telebot

import bot
bot.init()

import db
db.init()

import command_qrand
import command_rm
import command_sd
import command_nekto
import command_ass
import command_oleg

@bot.bot.message_handler(func=lambda _: True, content_types=['text', 'photo'])
def on_message(message: telebot.types.Message):
    db.save_message(message)

bot.bot.infinity_polling()