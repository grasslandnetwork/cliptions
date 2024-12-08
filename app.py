import os
import time
import base64
import requests
import subprocess
from datetime import datetime

# Define the API URL and headers for the Hugging Face Inference API
API_URL = "https://api-inference.huggingface.co/models/openai/clip-vit-base-patch32"
headers = {"Authorization": "Bearer " + os.environ['HF_TOKEN']}


def setup_temp_directory(directory_path):
    """
    Ensure the temporary directory exists.
    Args:
        directory_path (str): Path to the directory to create.
    """
    if not os.path.exists(directory_path):
        os.makedirs(directory_path)


def capture_snapshot(image_path):
    """
    Capture a snapshot from the Raspberry Pi camera using raspistill.

    Args:
        image_path (str): Path to save the captured image.
    
    Raises:
        Exception: If raspistill command fails.
    """
    try:
        # Use raspistill to capture an image
        subprocess.run(
            ["raspistill", "-o", image_path, "-w", "1280", "-h", "720", "-t", "1", "-n"],
            check=True,
        )
    except subprocess.CalledProcessError as e:
        raise Exception("Error capturing snapshot: {}".format(e))


def query(data):
    """
    Send a query to the Hugging Face API with the given image and parameters.

    Args:
        data (dict): Dictionary containing "image_path" and "parameters".
    
    Returns:
        dict: JSON response from the API.
    """
    try:
        # Read the image file as binary data
        with open(data["image_path"], "rb") as f:
            img = f.read()

        # Prepare the payload
        payload = {
            "parameters": data["parameters"],
            "inputs": base64.b64encode(img).decode("utf-8"),
        }

        # Send the POST request to the API
        response = requests.post(API_URL, headers=headers, json=payload)
        response.raise_for_status()
        return response.json()

    except Exception as e:
        print("Error during API query: {}".format(e))
        return None


# Main script logic
if __name__ == "__main__":
    temp_dir = "/tmp/realmir-logs-and-images"
    snapshot_path = os.path.join(temp_dir, "snapshot.jpg")  # Temporary file for the snapshot
    log_file = os.path.join(temp_dir, "app.log")  # Temporary file for logs

    # Ensure the temp directory exists
    setup_temp_directory(temp_dir)

    print("Starting the half-hour loop... Press Ctrl+C to stop.")
    try:
        while True:
            # Capture a snapshot
            try:
                capture_snapshot(snapshot_path)
            except Exception as e:
                print("Snapshot error: {}".format(e))
                continue

            # Query the API with the captured image
            output = query(
                {
                    "image_path": snapshot_path,
                    "parameters": {
                        "candidate_labels": [
                            "cat", "dog", "llama", "desk", "human",
                            "elizabeth", "man", "wellington", "abbey",
                        ]
                    },
                }
            )

            # Write the output to a log file with date and timestamp
            with open(log_file, "a") as log:
                timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                if output:
                    log.write("[{}] API Response: {}\n".format(timestamp, output))
                else:
                    log.write("[{}] Failed to get a valid response.\n".format(timestamp))

            # Wait for 30 minutes before capturing the next image
            time.sleep(1800)

    except KeyboardInterrupt:
        print("Loop stopped by user.")
