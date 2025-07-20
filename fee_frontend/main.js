// For MVP testing - simplified wallet simulation
console.log('🚀 main.js loaded successfully!');
const DEMO_MODE = true;

// State management
let provider = null;
let signer = null;
let userAddress = null;

// DOM elements
const connectButton = document.getElementById('connectWallet');
const twitterInput = document.getElementById('twitterHandle');
const statusDiv = document.getElementById('status');

// Event listeners
connectButton.addEventListener('click', handleConnectWallet);

function showStatus(message, type = 'info') {
    statusDiv.textContent = message;
    statusDiv.className = `status ${type}`;
    statusDiv.classList.remove('hidden');
}

function hideStatus() {
    statusDiv.classList.add('hidden');
}

async function handleConnectWallet() {
    try {
        // Validate Twitter handle
        const twitterHandle = twitterInput.value.trim();
        if (!twitterHandle) {
            showStatus('Please enter your Twitter username', 'error');
            return;
        }

        // Basic validation for Twitter handle
        if (!/^[a-zA-Z0-9_]{1,15}$/.test(twitterHandle)) {
            showStatus('Please enter a valid Twitter username (letters, numbers, and underscores only)', 'error');
            return;
        }

        connectButton.disabled = true;
        showStatus('Connecting to wallet...', 'info');

        // Connect wallet
        await connectWallet();
        
        if (!userAddress) {
            throw new Error('Failed to get wallet address');
        }

        showStatus('Wallet connected! Please sign the message to verify your identity...', 'info');

        // Create message to sign
        const message = `I am ${twitterHandle} on Twitter and I own this wallet address: ${userAddress}`;
        
        // Sign message
        showStatus('Please sign the message in your wallet...', 'info');
        const signature = await signMessage(message);

        // Send to backend
        showStatus('Verifying signature...', 'info');
        await submitToBackend(twitterHandle, userAddress, message, signature);

        showStatus('✅ Success! Your fee payment has been verified.', 'success');
        connectButton.textContent = 'Fee Paid Successfully';

    } catch (error) {
        console.error('Error:', error);
        showStatus(`Error: ${error.message}`, 'error');
        connectButton.disabled = false;
    }
}

async function connectWallet() {
    try {
        if (DEMO_MODE) {
            // Simulate wallet connection for MVP testing
            showStatus('🔄 Simulating wallet connection...', 'info');
            await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate delay
            
            // Generate a mock wallet address
            userAddress = '0x' + Math.random().toString(16).substring(2, 42).padStart(40, '0');
            showStatus('✅ Demo wallet connected!', 'info');
            return;
        }
        
        // Real wallet connection would go here when Web3Modal is properly configured
        throw new Error('Real wallet connection not configured yet');
        
    } catch (error) {
        throw error;
    }
}

async function signMessage(message) {
    try {
        if (DEMO_MODE) {
            // Simulate message signing for MVP testing
            showStatus('🔄 Simulating message signing...', 'info');
            await new Promise(resolve => setTimeout(resolve, 800)); // Simulate delay
            
            // Generate a mock signature (valid hex format)
            const mockSignature = `0x${'a'.repeat(130)}`;
            return mockSignature;
        }
        
        // Real message signing would go here when wallet is properly connected
        throw new Error('Real message signing not configured yet');
        
    } catch (error) {
        throw new Error(`Failed to sign message: ${error.message}`);
    }
}

async function submitToBackend(twitterHandle, walletAddress, message, signature) {
    try {
        const response = await fetch('/verify-payment', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                twitter_handle: twitterHandle,
                wallet_address: walletAddress,
                message: message,
                signature: signature
            })
        });

        if (!response.ok) {
            const errorData = await response.json().catch(() => ({ error: 'Unknown error' }));
            throw new Error(errorData.error || `HTTP ${response.status}`);
        }

        const result = await response.json();
        return result;

    } catch (error) {
        throw new Error(`Failed to verify payment: ${error.message}`);
    }
}

// Initialize the app
document.addEventListener('DOMContentLoaded', function() {
    if (DEMO_MODE) {
        showStatus('🧪 Demo mode: Wallet connection will be simulated for testing', 'info');
        setTimeout(() => hideStatus(), 3000); // Hide after 3 seconds
    }
}); 