import telebot
from bot import bot
from consts import BOT_ID, ADMIN_ID

@bot.message_handler(commands=['rm'])
def on_rm(message: telebot.types.Message):
    if not message.reply_to_message:
        return
    msg = message.reply_to_message
    if msg.from_user.id != BOT_ID:
        return
    if message.from_user.id != ADMIN_ID:
        return
    msg = message.reply_to_message
    bot.delete_message(msg.chat.id, msg.id)