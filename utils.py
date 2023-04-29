import base64
import io
from PIL import Image

def img2base64(img: bytes) -> str:
    img = Image.open(io.BytesIO(img))
    with io.BytesIO() as bytes:
        img.save(bytes, format='PNG')
        b64 = str(base64.b64encode(bytes.getvalue()), 'utf-8')
        return f'data:image/png;base64,{b64}'