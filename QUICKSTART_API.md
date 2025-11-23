# Quick Start: VisualSign API for EigenLayer

Deploy the EigenLayer transaction parser as a REST API in under 5 minutes!

## 🚀 One-Command Deploy

```bash
docker-compose up -d
```

That's it! Your API is now running on `http://localhost:3000`

## ✅ Test It

```bash
# Health check
curl http://localhost:3000/health

# Get API info
curl http://localhost:3000/info

# Parse a transaction
curl -X POST http://localhost:3000/parse \
  -H "Content-Type: application/json" \
  -d '{
    "transaction": "0xf8890c84060d173b830ea60094858646372cc42e1a627fce94aa7a7033e7cf075a80b864e7a050aa00000000000000000000000093c4b944d05dfe6df7645a86cd2206016c51564d000000000000000000000000ae7ab96520de3a18e5e111b5eaab095312d7fe8400000000000000000000000000000000000000000000000000028b30699cdc00018080",
    "chain_id": 1
  }'
```

## 📊 Example Response

```json
{
  "payload": {
    "title": "EigenLayer: Deposit Into Strategy",
    "subtitle": "Deposit 1.5 stETH into stETH Strategy",
    "fields": [
      {
        "type": "PreviewLayout",
        "condensed": {
          "fields": [
            {
              "label": "Strategy",
              "address": "0x93c4...564d",
              "name": "stETH Strategy",
              "asset_label": "stETH",
              "badge_text": "Verified"
            },
            {
              "label": "Amount",
              "amount": "1.5",
              "abbreviation": "stETH"
            }
          ]
        },
        "expanded": {
          "fields": [
            // Full details with dividers, annotations, etc.
          ]
        }
      }
    ]
  }
}
```

## 🛠️ Configuration

### Custom Port

```bash
# Edit docker-compose.yml
ports:
  - "8080:8080"
environment:
  - PORT=8080
```

### Enable Logging

```bash
# Edit docker-compose.yml
environment:
  - RUST_LOG=debug
```

### With Nginx Reverse Proxy

```bash
docker-compose --profile with-nginx up -d
```

## 📚 Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| GET | `/info` | API information |
| POST | `/parse` | Parse transaction |
| POST | `/api/v1/parse` | Parse transaction (versioned) |

## 🔧 Management

```bash
# View logs
docker-compose logs -f

# Stop
docker-compose down

# Restart
docker-compose restart

# Scale (3 instances)
docker-compose up -d --scale visualsign-api=3
```

## 🌐 Production Deploy

### Deploy to Cloud

**DigitalOcean / AWS / GCP:**
1. Clone repo on your server
2. Run `docker-compose up -d`
3. Configure firewall for port 3000
4. Done!

**With Domain:**
1. Point DNS A record to your server IP
2. Use nginx profile: `docker-compose --profile with-nginx up -d`
3. Add SSL with Let's Encrypt (optional)

### Environment Variables

```bash
# .env file
PORT=3000
RUST_LOG=info
```

## 📖 Full Documentation

See [api-server/README.md](api-server/README.md) for:
- Complete API reference
- All supported methods (60 total)
- JavaScript/Python examples
- Production deployment guide
- Performance tuning

## 🎯 Features

- ✅ 100% EigenLayer coverage (60 methods)
- ✅ Condensed + Expanded views
- ✅ 14 LST strategies with metadata
- ✅ Human-readable amounts (ETH not wei)
- ✅ Badges: Operator, AVS, Verified
- ✅ Warnings and annotations
- ✅ Docker ready
- ✅ CORS enabled
- ✅ Health checks
- ✅ Rate limiting (with nginx)

## 💡 Use Cases

- Wallet transaction preview
- DApp transaction decoding
- Blockchain explorer
- Transaction monitoring
- Compliance tools
- User education

## 🆘 Troubleshoot

```bash
# Check if running
docker ps

# View logs
docker-compose logs visualsign-api

# Test health
curl http://localhost:3000/health

# Rebuild
docker-compose build --no-cache
docker-compose up -d
```

## 📦 What's Included

- REST API server (Rust/Axum)
- Docker configuration
- Docker Compose setup
- Nginx reverse proxy config
- Health checks
- CORS support
- Rate limiting
- Comprehensive docs

## ⚡ Quick Examples

### JavaScript
```javascript
const response = await fetch('http://localhost:3000/parse', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ transaction: '0x...' })
});
const data = await response.json();
```

### Python
```python
import requests
response = requests.post('http://localhost:3000/parse', 
    json={'transaction': '0x...'})
result = response.json()
```

### cURL
```bash
curl -X POST http://localhost:3000/parse \
  -H "Content-Type: application/json" \
  -d '{"transaction": "0x..."}'
```

---

**Ready to parse EigenLayer transactions!** 🎉

For advanced usage, see [api-server/README.md](api-server/README.md)
