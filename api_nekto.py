import os
os.environ['PROTOCOL_BUFFERS_PYTHON_IMPLEMENTATION'] = 'python'

import anticaptchaofficial.recaptchav2proxyless
import anticaptchaofficial.imagecaptcha
import random
import socketio
import time
import uuid
import ed25519
import hashlib
import blowfish
import base64
import requests
import string
import checkin_pb2
from enum import Enum, auto
from consts import ANTICAPTCHA_TOKEN

rec_solver = anticaptchaofficial.recaptchav2proxyless.recaptchaV2Proxyless()
rec_solver.set_key(ANTICAPTCHA_TOKEN)
rec_solver.set_website_url('https://nekto.me')

img_solver = anticaptchaofficial.imagecaptcha.imagecaptcha()
img_solver.set_key(ANTICAPTCHA_TOKEN)

def get_android_id():
	# most minimal checkin request possible
	cr = checkin_pb2.CheckinRequest()
	cr.androidId= 0
	cr.checkin.build.fingerprint = "google/razor/flo:5.0.1/LRX22C/1602158:user/release-keys"
	cr.checkin.build.hardware = "flo"
	cr.checkin.build.brand = "google"
	cr.checkin.build.radio = "FLO-04.04"
	cr.checkin.build.clientId = "android-google"
	cr.checkin.build.sdkVersion = 21
	cr.checkin.lastCheckinMs = 0
	cr.locale = "en"
	cr.macAddress.append("".join(random.choice("ABCDEF0123456789") for _ in range(12)))
	cr.meid = "".join(random.choice("0123456789") for _ in range(15))
	cr.timeZone = "Europe/London"
	cr.version = 3
	cr.otaCert.append("--no-output--")
	cr.macAddressType.append("wifi")
	cr.fragment = 0
	cr.userSerialNumber = 0

	data = cr.SerializeToString()
	headers = {"Content-type": "application/x-protobuffer",
    	       "Accept-Encoding": "gzip",
        	   "User-Agent": "Android-Checkin/2.0 (vbox86p JLS36G); gzip"}
	r = requests.post("https://android.clients.google.com/checkin", headers=headers, data=data)

	if r.status_code == 200:
		cresp = checkin_pb2.CheckinResponse()
		cresp.ParseFromString(r.content)
		android_id = cresp.androidId
		security_token = cresp.securityToken
		return android_id, security_token

	else:
		print(r.text)

class NektoApi:
    class State(Enum):
        Disconnected = auto()
        Connecting = auto()
        Connected = auto()
        Logined = auto()
        CaptchaRequired = auto()
        CaptchaSolved = auto()
        Searching = auto()
        InChat = auto()

    def __init__(self):
        self.state = NektoApi.State.Disconnected
        self.device_name = 'ASUS_T00J'
        self.sig_key = 'MWUwOGE5MDNhZWY5YzNhNzIxNTEwYjY0ZWM3NjRkMDFkM2QwOTRlYjk1NDE2MWI2MjU0NGVhOGYxODdiNTk1Mw=='
        self.auth_token = None
        self.ins_key = None
        self.eddsa_key = None
        self.push_token = None
        self.vending = None
        self.id = None
        self.dialog_id = None
        self.companion_id = None
        self.on_state = None
        self.on_end = None
        self.on_message = None
        self.sio = socketio.Client(engineio_logger=True, logger=True)
        self.sio.on('connect', self._on_connect)
        self.sio.on('notice', self._on_notice)
        self.sio.on('disconnect', self._on_disconnect)

    def _state(self, state: State):
        if self.state != state:
            self.state = state
            self.on_state(self)

    def _send(self, command):
        self.sio.emit('action', command)

    def _register(self):
        fid = ''.join([random.choice(string.ascii_letters) for _ in range(23)])
        installation = requests.post('https://firebaseinstallations.googleapis.com/v1/projects/nekto-me/installations', json={
            'appId': '1:962390669690:android:a184f1d8b3b493ad',
            'authVersion': 'FIS_v2',
            'fid': fid,
            'sdkVersion': 'a:17.0.1',
        }, headers={'x-goog-api-key': 'AIzaSyDjcoBVU2NNoCejlYTZu-GnNBtvm-brhQw'}).json()
        fid = installation['fid']
        device, security = get_android_id()
        push_token = requests.post('https://android.apis.google.com/c2dm/register3', headers={
            'Authorization': f'AidLogin {device}:{security}',
            'app': 'com.nektome.talk',
            'gcm_ver': '19056022',
            'User-Agent': 'Android-GCM/1.5 (OnePlus5 NMF26X)',
        }, data={
            'X-subtype': '962390669690',
            'sender': '962390669690',
            'X-app_ver': '195',
            'X-osv': '25',
            'X-cliv': 'fcm-23.0.2',
            'X-gmsv': '19056022',
            'X-appid': fid,
            'X-scope': '*',
            'X-Goog-Firebase-Installations-Auth': installation['authToken']['token'],
            'X-gmp_app_id': '1:962390669690:android:a184f1d8b3b493ad',
            'X-firebase-app-name-hash': 'R1dAH9Ui7M-ynoznwBdw01tLxhI',
            'X-app_ver_name': '4.0.8',
            'app': 'com.nektome.talk',
            'device': f'{device}',
            'app_ver': '195',
            'gcm_ver': '19056022',
            'plat': '0',
            'cert': '5d08264b44e0e53fbccc70b4f016474cc6c5ab5c',
            'target_ver': '31',
        }).text.split('=', 1)[1]

        device_id = str(uuid.uuid4())
        seed = hashlib.sha256(device_id.encode('utf-8')).digest()
        key = ed25519.SigningKey(seed).sign(seed).hex()

        #vending = 'com.bluestacks.BstCommandProcessor'
        #vending = 'empty-huawei'
        vending = 'empty-google'
        cipher = blowfish.Cipher(('com.nektome.talk' + device_id).encode('utf-8'))
        padding = (8 - (len(vending) % 8)) % 8
        ins_key = base64.b64encode(b''.join(cipher.encrypt_ecb((vending + '\x06' * padding).encode('utf-8')))).decode('utf-8').replace('/', '\\/')

        return (device_id, ins_key, key, push_token, vending)

    def _on_connect(self):
        print('connection established')
        self._state(NektoApi.State.Connected)

        if not self.auth_token:
            device_id, ins_key, key, push_token, vending = self._register()
            self.ins_key = ins_key
            self.eddsa_key = key
            self.push_token = push_token
            self.vending = vending
            self._send({
                'deviceId': device_id,
                'deviceName': self.device_name,
                'deviceType': 1,
                'insKey': ins_key,
                'key': key,
                'push': push_token,
                'pType': 101,
                'sigKey': self.sig_key,
                'vending': vending,
                'action': 'auth.getToken'
            })
        else:
            self._send({
                'insKey': self.ins_key,
                'key': self.eddsa_key,
                'pushToken': self.push_token,
                'pType': 101,
                'sigKey': self.sig_key,
                'token': self.auth_token,
                'vending': self.vending,
                'action': 'auth.sendToken'
            })

    def _on_notice(self, data):
        type = data['notice']
        data = data['data']
        if type == 'auth.successToken':
            self.auth_token = data['tokenInfo']['authToken']
            self.id = data['id']
            self._send({'action': 'online.track', 'on': True})
            self._state(NektoApi.State.Logined)
        elif type == 'error.code':
            print(data)
            code = data['id']
            if code == 600:
                self._state(NektoApi.State.CaptchaRequired)
                if 'captchaImage' in data['additional']:
                    path = data['additional']['captchaImage']
                    fn = path.split('/')[-1]
                    with open(fn, 'wb') as f:
                        f.write(requests.get(path).content)
                    solution = img_solver.solve_and_return_solution(fn)
                    os.remove(fn)
                    self._send({'action': 'captcha.verify', 'solution': solution})
                else:
                    rec_solver.set_website_key(data['additional']['publicKey'])
                    solution = rec_solver.solve_and_return_solution()
                    self._send({'action': 'captcha.verify', 'solution': solution, 'hard': False})
        elif type == 'captcha.verify':
            self._state(NektoApi.State.CaptchaSolved)
        elif type == 'dialog.opened':
            self.dialog_id = data['id']
            companion = data['interlocutors']
            companion.remove(self.id)
            self.companion_id = companion[0]
            self._state(NektoApi.State.InChat)
        elif type == 'dialog.closed':
            self.on_end(self)
            self._state(NektoApi.State.Logined)
            self.dialog_id = None
            self.companion_id = None
        elif type == 'messages.new':
            self.on_message(self, data['senderId'], data['message'])
            if data['senderId'] != self.id:
                self._send({'action': 'anon.readMessages', 'dialogId': self.dialog_id, 'lastMessageId': data['id']})
        else:
            print(type, data)

    def _on_disconnect(self):
        self.dialog_id = None
        self.companion_id = None
        self._state(NektoApi.State.Disconnected)
        print('disconnected from server')

    def connect(self):
        if self.state == NektoApi.State.Disconnected:
            self._state(NektoApi.State.Connecting)
            hh = {
                'Android-Language': 'en',
                'App-Android-Code': '195',
                'App-Android-Version': '4.0.8',
                'NektoMe-Chat-Version': '1',
                'Sec-WebSocket-Server': 'dmVyc2lvbi0xMDAx',
                'Sec-WebSocket-STR': 'GL',
                'User-Agent': f'NektoMe195/2.1.0 (Linux; U; Android 7.1.1; {self.device_name} Build/NMF26X)',
            }
            self.sio.connect('wss://im.nekto.me', transports='websocket', headers=hh)

    def end(self):
        self._send({'action': 'anon.leaveDialog', 'dialogId': self.dialog_id})
        self._send({'action': 'search.sendOut'})

    def search(self):
        print('Starting search')
        self._state(NektoApi.State.Searching)
        self._send({'action': 'search.run'})

    def send(self, message):
        random_id = f'{self.id}_{round(time.time() * 1000)}{random.random()}'
        self._send({'action': 'anon.message', 'dialogId': self.dialog_id, 'message': message, 'randomId': random_id, 'fileId': None})
