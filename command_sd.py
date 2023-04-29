import telebot
import datetime
import requests
import io
import traceback
import base64
import db
from PIL import Image
from bot import bot
from consts import WHITELIST, SD_TIMEOUT_LIST, SD_TIMEOUT_MESSAGE, SD_TIMEOUT, SD_URL
from utils import img2base64

LAST_SD = {}

def sd_filter(message: telebot.types.Message):
    text = message.text or message.caption
    if text and text.split(' ', 1)[0] == '/sd':
        message.text = text.split('/sd', 1)
        if len(message.text) > 1:
            message.text = message.text[1].strip()
        else:
            if message.reply_to_message and message.reply_to_message.text:
                message.text = message.reply_to_message.text
            else:
                message.text = ''
        return True
    return False

@bot.message_handler(func=sd_filter, content_types=['text', 'photo'])
def on_sd(message: telebot.types.Message):
    if message.chat.id not in WHITELIST:
        return

    PHOTO_SIZE = 512

    if message.chat.id in SD_TIMEOUT_LIST and message.chat.id in LAST_SD and (datetime.datetime.now() - LAST_SD[message.chat.id]).total_seconds() < SD_TIMEOUT:
        bot.reply_to(message, SD_TIMEOUT_MESSAGE)
        return

    LAST_SD[message.chat.id] = datetime.datetime.now()

    def set_checkpoint(name: str):
        requests.post(f'{SD_URL}/sdapi/v1/options', json={
            'sd_model_checkpoint': name
        })

    MAIN21 = 'v2-1_768-ema-pruned-fp16.safetensors [370234eabb]'
    MAIN15 = 'deliberate_v2.safetensors [9aba26abdf]'
    DEPTH = '512-depth-ema-fp16.safetensors [1edad03947]'

    try:
        msg = message
        if message.reply_to_message and message.reply_to_message.photo:
            msg = message.reply_to_message
        if msg.photo and len(msg.photo) > 0:
            set_checkpoint(DEPTH)
            file_info = bot.get_file(msg.photo[-1].file_id)
            response = requests.post(f'{SD_URL}/sdapi/v1/img2img', json={
                'width': PHOTO_SIZE,
                'height': PHOTO_SIZE,
                'steps': 20,
                'resize_mode': 1,
                'denoising_strength': 0,
                'script_name': 'Depth aware img2img mask',
                'script_args': [False, 0, True, PHOTO_SIZE, PHOTO_SIZE, False, 1, True, False, False, False],
                'init_images': [img2base64(bot.download_file(file_info.file_path))],
            }).json()
            with io.BytesIO() as depth:
                Image.open(io.BytesIO(base64.b64decode(response['images'][1]))).convert('L').save(depth, 'PNG')
                response = requests.post(f'{SD_URL}/sdapi/v1/txt2img', json={
                    'prompt': message.text,
                    'sampler_name': 'Euler a',
                    'width': PHOTO_SIZE,
                    'height': PHOTO_SIZE,
                    'steps': 20,
                    'resize_mode': 1,
                    'denoising_strength': 0,
                    'script_name': 'Custom Depth Images (input/output)',
                    'script_args': [img2base64(depth.getvalue()).split(',', 1)[1], False, None, None, False],
                }).json()
                if 'images' in response:
                    print(len(response['images']))
                    img = io.BytesIO(base64.b64decode(response['images'][0].split(",", 1)[0]))
                    img.name = 'image.png'
                    db.save_message(bot.send_photo(message.chat.id, reply_to_message_id=message.id, photo=img, has_spoiler=True))
        else:
            set_checkpoint(MAIN15)
            response = requests.post(f'{SD_URL}/sdapi/v1/txt2img', json={
                'steps': 20,
                'sampler_name': 'Euler a',
                'width': PHOTO_SIZE,
                'height': PHOTO_SIZE,
                #'enable_hr': True,
                'hr_upscaler': 'Latent',
                'denoising_strength': 0.7,
                'prompt': message.text,
                'negative_prompt': '(deformed iris, deformed pupils, semi-realistic, cgi, 3d, render, sketch, cartoon, drawing, anime:1.4), text, close up, cropped, out of frame, worst quality, low quality, jpeg artifacts, ugly, duplicate, morbid, mutilated, extra fingers, mutated hands, poorly drawn hands, poorly drawn face, mutation, deformed, blurry, dehydrated, bad anatomy, bad proportions, extra limbs, cloned face, disfigured, gross proportions, malformed limbs, missing arms, missing legs, extra arms, extra legs, fused fingers, too many fingers, long neck',
            }).json()
            if 'images' in response:
                img = io.BytesIO(base64.b64decode(response['images'][0].split(",", 1)[0]))
                img.name = 'image.png'
                db.save_message(bot.send_photo(message.chat.id, reply_to_message_id=message.id, photo=img, has_spoiler=True))
    except Exception as e:
        traceback.print_exception(e)