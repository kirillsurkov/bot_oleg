import telebot
import assistant
import traceback
from huggingface_hub.inference_api import InferenceApi
from bot import bot
from consts import HF_TOKEN

inference = InferenceApi(repo_id='OpenAssistant/oasst-sft-4-pythia-12b-epoch-3.5', token=HF_TOKEN)

def ass_filter(message: telebot.types.Message):
    text = message.text or message.caption
    if text and text.split(' ', 1)[0] == '/ass':
        message.text = text.split('/ass', 1)
        if len(message.text) > 1:
            message.text = message.text[1].strip()
        else:
            message.text = ''
        return True
    return False

def get_ass(messages: list):
    prompt = '\n'.join([f'<|{"assistant" if m[1] else "prompter"}|>{m[2]}: {m[3]}<|endoftext|>' for m in messages if len(m[3].strip()) > 0] + ['<|assistant|>'])
    print(prompt)
    res = ''
    while True:
        try:
            response = inference(inputs=prompt + res)
            print(response)
            generated = response[0]['generated_text'][len(prompt + res):]
            res += generated
            if len(generated) == 0:
                break
        except Exception as e:
            traceback.print_exception(e)
            break
    return res

@bot.message_handler(func=ass_filter, content_types=['text', 'photo'])
def on_mention(message: telebot.types.Message):
    assistant.do_reply(message, get_ass, 10)