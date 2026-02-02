import time
import requests
from utils import get_common_headers, get_timestamp

def build_inquiry_payload(params):
    """
    根据传入的参数构建完整的询价负载。
    """
    timestamp = get_timestamp()
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

def create_new_inquiry(input_data, token=None):
    url = "https://cust.adentrd.com/trs-cust-gateway/trs-option-core/quote-order/createNew"
    
    # 构建最终负载
    body_data = build_inquiry_payload(input_data)
    headers = get_common_headers(token)
    
    # 额外的 Header 信息 (保持原样)
    headers.update({
        "Accept-Encoding": "gzip, deflate, br, zstd",
        "Accept-Language": "zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6",
        "Sec-Ch-Ua": '"Not(A:Brand";v="8", "Chromium";v="144", "Microsoft Edge";v="144"',
        "Sec-Ch-Ua-Mobile": "?0",
        "Sec-Ch-Ua-Platform": '"Windows"',
        "Sec-Fetch-Dest": "empty",
        "Sec-Fetch-Mode": "cors",
        "Sec-Fetch-Site": "same-site",
    })

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
