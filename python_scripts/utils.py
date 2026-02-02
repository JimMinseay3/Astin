import sys
import io
import time

# 强制标准输入输出使用 UTF-8 编码，防止 Windows 下的乱码问题
def setup_utf8():
    if sys.stdout.encoding != 'utf-8':
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    if sys.stdin.encoding != 'utf-8':
        sys.stdin = io.TextIOWrapper(sys.stdin.buffer, encoding='utf-8')

def get_auth_header(token):
    if not token:
        return ""
    if token.startswith("Bearer "):
        return token
    return f"Bearer {token}"

def get_common_headers(token):
    return {
        "Accept": "application/json, text/plain, */*",
        "Authorization": get_auth_header(token),
        "Content-Type": "application/json;charset=UTF-8",
        "Origin": "https://gray.adentrd.com",
        "Referer": "https://gray.adentrd.com/",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36 Edg/144.0.0.0",
        "User-Language": "CN"
    }

def get_timestamp():
    return int(time.time() * 1000)
