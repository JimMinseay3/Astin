import requests
from utils import get_common_headers, get_timestamp

def fetch_quote_list(page=1, page_size=10, security_code="", token=None):
    url = "https://cust.adentrd.com/trs-cust-gateway/trs-option-core/quote-order/list"
    
    timestamp = get_timestamp()
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
    
    headers = get_common_headers(token)

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
