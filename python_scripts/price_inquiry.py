import sys
import json
import requests
import time
import io

# 强制标准输入输出使用 UTF-8 编码，防止 Windows 下的乱码问题
if sys.stdout.encoding != 'utf-8':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
if sys.stdin.encoding != 'utf-8':
    sys.stdin = io.TextIOWrapper(sys.stdin.buffer, encoding='utf-8')

def build_inquiry_payload(params):
    """
    根据传入的参数构建完整的询价负载。
    """
    timestamp = int(time.time() * 1000)
    current_time_str = time.strftime("%Y-%m-%d %H:%M:%S", time.localtime())
    
    # 提取基本参数
    securities = params.get("securities", [])
    if not securities:
        # 兼容旧版本
        securities = [{
            "securityCode": params.get("securityCode", "000001"),
            "securityName": params.get("securityName", "平安银行")
        }]
    
    # 名义本金处理 (从 万 转换为 元)
    principal_val = params.get("notionalPrincipal", 500)
    if principal_val == 'custom':
        principal_val = params.get("customNotional", 500)
    notional_principal = int(principal_val) * 10000
    
    date_limits = params.get("dateLimits", ["M1", "M2", "M3"])
    structures = params.get("structures", ["S100", "S103", "S105"])
    
    special_reqs = params.get("specialRequirements", [])
    triple_knockout = "Y" if "tripleKnockout" in special_reqs else "N"
    t_one_execute = "Y" if "tOneExecute" in special_reqs else "N"
    is_bonus_adjust = "1" if "dividendAdjust" in special_reqs else "0"
    
    # 构建证券信息明细 (securityCodes)
    security_codes_payload = []
    for s in securities:
        code = s.get("securityCode")
        name = s.get("securityName")
        security_codes_payload.append({
            "id": str(int(time.time() * 1000000)), # 简单生成一个 ID
            "createTime": current_time_str,
            "lastModifyTime": current_time_str,
            "securityCode": code,
            "securityName": name,
            "exchangeCode": "SZ.QFII" if code.startswith("00") or code.startswith("30") else "SH.QFII",
            "securityStatus": "NORMAL",
            "listingDate": 19910403,
            "endDate": 19910403,
            "buyVolumeUnit": 100,
            "sellVolumeUnit": 1,
            "tickSize": 0.01,
            "version": 2173,
            "securityEngName": code,
            "securityType": "STOCK",
            "securityVariety": "A_MAIN_BOARD",
            "description": None,
            "extMargin": 1,
            "extHotSpot": 0,
            "extCirculationMarketCap": 210356933634.56,
            "securityCanUse": 1,
            "securityCodeVo": f"{code}.SZ" if code.startswith("00") or code.startswith("30") else f"{code}.SH"
        })

    return {
        "requestId": timestamp,
        "requestNo": timestamp,
        "securityCodes": security_codes_payload,
        "structures": structures,
        "notionalPrincipals": [notional_principal],
        "dateLimits": date_limits,
        "tripleKnockout": triple_knockout,
        "tOneExecute": t_one_execute,
        "isBonusAdjustInitPrice": is_bonus_adjust
    }

def get_auth_header(token):
    if not token:
        return "Bearer MrlW2aF3yGGLRywpU3eijcYztNduwbIh"
    if token.startswith("Bearer "):
        return token
    return f"Bearer {token}"

def fetch_security_pool(input_query="", token=None):
    url = "https://cust.adentrd.com/trs-cust-gateway/trs-option-core/quote-order/security"
    
    timestamp = int(time.time() * 1000)
    body_data = {
        "requestId": timestamp,
        "requestNo": timestamp,
        "page": 1,
        "pageSize": 10,
        "input": input_query
    }
    
    headers = {
        "Accept": "application/json, text/plain, */*",
        "Authorization": get_auth_header(token),
        "Content-Type": "application/json;charset=UTF-8",
        "Origin": "https://gray.adentrd.com",
        "Referer": "https://gray.adentrd.com/",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36 Edg/144.0.0.0",
        "User-Language": "CN"
    }

    try:
        response = requests.post(url, headers=headers, json=body_data, timeout=10)
        response.raise_for_status()
        return {
            "success": True,
            "data": response.json(),
            "status_code": response.status_code
        }
    except Exception as e:
        return {
            "success": False,
            "error": str(e)
        }

def create_new_inquiry(input_data, token=None):
    url = "https://cust.adentrd.com/trs-cust-gateway/trs-option-core/quote-order/createNew"
    
    # 构建最终负载
    body_data = build_inquiry_payload(input_data)
    headers = {
        "Accept": "application/json, text/plain, */*",
        "Accept-Encoding": "gzip, deflate, br, zstd",
        "Accept-Language": "zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6",
        "Authorization": get_auth_header(token),
        "Content-Type": "application/json;charset=UTF-8",
        "Origin": "https://gray.adentrd.com",
        "Referer": "https://gray.adentrd.com/",
        "Sec-Ch-Ua": '"Not(A:Brand";v="8", "Chromium";v="144", "Microsoft Edge";v="144"',
        "Sec-Ch-Ua-Mobile": "?0",
        "Sec-Ch-Ua-Platform": '"Windows"',
        "Sec-Fetch-Dest": "empty",
        "Sec-Fetch-Mode": "cors",
        "Sec-Fetch-Site": "same-site",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36 Edg/144.0.0.0",
        "User-Language": "CN"
    }

    try:
        response = requests.post(url, headers=headers, json=body_data, timeout=10)
        response.raise_for_status()
        return {
            "success": True,
            "data": response.json(),
            "status_code": response.status_code
        }
    except Exception as e:
        return {
            "success": False,
            "error": str(e)
        }

def mock_search(query):
    # Keep the mock search for UI testing
    return [
        {"site": "系统 A", "price": "￥1,299", "stock": "有货", "update_time": "刚刚"},
        {"site": "系统 B", "price": "￥1,250", "stock": "有货", "update_time": "5分钟前"}
    ]

def fetch_quote_list(page=1, page_size=10, security_code="", token=None):
    url = "https://cust.adentrd.com/trs-cust-gateway/trs-option-core/quote-order/list"
    
    timestamp = int(time.time() * 1000)
    body_data = {
        "requestId": timestamp,
        "requestNo": timestamp,
        "page": page,
        "pageSize": page_size,
        "securityCode": security_code,
        "userKey": "",
        "pageNum": page,
        "limit": page_size
    }
    
    headers = {
        "Accept": "application/json, text/plain, */*",
        "Authorization": get_auth_header(token),
        "Content-Type": "application/json;charset=UTF-8",
        "Origin": "https://gray.adentrd.com",
        "Referer": "https://gray.adentrd.com/",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36 Edg/144.0.0.0",
        "User-Language": "CN"
    }

    try:
        response = requests.post(url, headers=headers, json=body_data, timeout=10)
        response.raise_for_status()
        return {
            "success": True,
            "data": response.json(),
            "status_code": response.status_code
        }
    except Exception as e:
        return {
            "success": False,
            "error": str(e)
        }

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(json.dumps({"error": "No action provided"}))
        sys.exit(1)

    action = sys.argv[1]
    
    if action == "search":
        query = sys.argv[2] if len(sys.argv) > 2 else ""
        token = sys.argv[3] if len(sys.argv) > 3 else None
        result = fetch_security_pool(query, token)
        print(json.dumps(result, ensure_ascii=False))
    elif action == "create":
        # 从 stdin 读取 body，避免 Windows 命令行转义问题
        try:
            token = sys.argv[2] if len(sys.argv) > 2 else None
            body_str = sys.stdin.read().strip()
            
            if not body_str:
                body_str = "{}"
            
            # 尝试解析 JSON
            body_data = json.loads(body_str)
            result = create_new_inquiry(body_data, token)
            print(json.dumps(result, ensure_ascii=False))
        except Exception as e:
            # 如果解析失败，返回详细的错误信息
            import traceback
            error_detail = traceback.format_exc()
            print(json.dumps({
                "success": False, 
                "error": f"Invalid JSON body: {str(e)}",
                "detail": error_detail,
                "received_content": body_str if 'body_str' in locals() else "None"
            }, ensure_ascii=False))
    elif action == "list":
        page = int(sys.argv[2]) if len(sys.argv) > 2 else 1
        page_size = int(sys.argv[3]) if len(sys.argv) > 3 else 10
        security_code = sys.argv[4] if len(sys.argv) > 4 else ""
        token = sys.argv[5] if len(sys.argv) > 5 else None
        result = fetch_quote_list(page, page_size, security_code, token)
        print(json.dumps(result, ensure_ascii=False))
    else:
        print(json.dumps({"error": f"Unknown action: {action}"}))
