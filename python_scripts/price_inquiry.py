import sys
import json
from utils import setup_utf8
from security_api import fetch_security_pool
from inquiry_api import create_new_inquiry
from history_api import fetch_quote_list

# 初始化 UTF-8 设置
setup_utf8()

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
            # 改为读取单行，配合 Rust 端发送换行符，避免 sys.stdin.read() 导致无限等待 EOF
            body_str = sys.stdin.readline().strip()
            
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
