#!/bin/bash

# RealMIR Discord Bot Runner Script

echo "🤖 RealMIR Discord Bot Runner"
echo "=============================="

# Check if .env file exists
if [ ! -f ".env" ]; then
    echo "📄 Creating .env file from template..."
    cp .env.example .env
    echo "⚠️  Please edit .env file and add your DISCORD_TOKEN"
    echo "   You can get a token from: https://discord.com/developers/applications"
    exit 1
fi

# Check if DISCORD_TOKEN is set
if ! grep -q "DISCORD_TOKEN=.*[^[:space:]]" .env; then
    echo "❌ Error: DISCORD_TOKEN not set in .env file"
    echo "   Please edit .env and add your Discord bot token"
    exit 1
fi

# Check if config directory exists
if [ ! -d "config" ]; then
    echo "📁 Creating config directory..."
    mkdir -p config
fi

# Check if config file exists
if [ ! -f "config/realmir.yaml" ]; then
    echo "⚠️  Config file config/realmir.yaml not found, but bot will use defaults"
fi

echo "🔨 Building Discord bot..."
if cargo build --features discord --bin realmir_discord_bot; then
    echo "✅ Build successful!"
    echo ""
    echo "🚀 Starting RealMIR Discord Bot..."
    echo "   Press Ctrl+C to stop"
    echo ""
    
    # Load environment variables and run
    export $(cat .env | xargs)
    cargo run --features discord --bin realmir_discord_bot
else
    echo "❌ Build failed! Please check the error messages above."
    exit 1
fi