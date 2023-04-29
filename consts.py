import os
import dotenv
dotenv.load_dotenv()

BOT_TOKEN = os.environ.get('BOT_TOKEN')
OPENAI_TOKEN = os.environ.get('OPENAI_TOKEN')
ANTICAPTCHA_TOKEN = os.environ.get('ANTICAPTCHA_TOKEN')
HF_TOKEN = os.environ.get('HF_TOKEN')

ADMIN_ID = int(os.environ.get('ADMIN_ID'))
BOT_ID = int(os.environ.get('BOT_ID'))
BOT_ALIAS = os.environ.get('BOT_ALIAS')

OLEG_PROMPT = os.environ.get('OLEG_PROMPT')

WHITELIST = [ADMIN_ID] + [int(x) for x in os.environ.get('WHITELIST').split(',')]

SD_TIMEOUT = int(os.environ.get('SD_TIMEOUT'))
SD_TIMEOUT_LIST = [int(x) for x in os.environ.get('SD_TIMEOUT_LIST').split(',')]
SD_TIMEOUT_MESSAGE = os.environ.get('SD_TIMEOUT_MESSAGE')
SD_URL = os.environ.get('SD_URL')

MEMORY_SIZE = int(os.environ.get('MEMORY_SIZE'))