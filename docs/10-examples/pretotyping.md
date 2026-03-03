# Pretotyping

([No, it's not a typo.](http://www.pretotyping.org/))

Opal aims to make simple web applications as concise as possible:

**Python (Flask):**
```python
from flask import Flask
app = Flask(__name__)

@app.route("/")
def hello():
    return "Hello World!"

if __name__ == "__main__":
    app.run()
```

**Opal equivalent:**
```opal
import OpalWeb

app = OpalWeb.App("app name")

@get "/" do
  "Hello world!"
end

app.run!
```
