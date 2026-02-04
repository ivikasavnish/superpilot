from flask import Flask
import os

app = Flask(__name__)

@app.route('/')
def hello():
    return 'Hello from Python with Superpilot!'

@app.route('/health')
def health():
    return {'status': 'healthy'}

if __name__ == '__main__':
    # Debug mode should be False in production
    debug_mode = os.getenv('FLASK_DEBUG', 'False').lower() == 'true'
    app.run(host='0.0.0.0', port=8081, debug=debug_mode)
