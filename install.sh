#!/bin/bash

LATEST_TAG=$(curl -s "https://api.github.com/repos/Luganodes/ftsov2mon/tags" | jq -r '.[0].name')
DESTINATION_DIR="/usr/local/bin"
DOWNLOAD_LINK="https://github.com/Luganodes/ftsov2mon/releases/download/$LATEST_TAG/ftsov2mon"
BINARY_NAME="ftsov2mon"
SERVICE_FILE="/etc/systemd/system/ftsov2mon.service"

# Function to check OS type
check_os_type() {
  case "$(uname -s)" in
    Linux*)     OS="Linux" ;;
    Darwin*)    OS="MacOS"; echo "MacOS is not supported. Exiting."; exit 1 ;;
    *)          echo "Unsupported OS"; exit 1 ;;
  esac
}

# Function to install the binary
install_binary() {
  # Check if the user is root or has sudo privileges
  if [ "$EUID" -ne 0 ]; then
    echo "Please run this script with sudo or as root."
    exit 1
  fi

  # Check if the destination directory exists
  if [ ! -d "$DESTINATION_DIR" ]; then
    echo "Creating $DESTINATION_DIR..."
    mkdir -p "$DESTINATION_DIR"
  fi

  # Download the binary to the destination directory
  echo "Downloading $BINARY_NAME to $DESTINATION_DIR..."
  wget -q "$DOWNLOAD_LINK" -O "$DESTINATION_DIR/$BINARY_NAME"
  if [ $? -ne 0 ]; then
    echo "Failed to download the binary. Check the URL or network connection."
    exit 1
  fi

  chmod +x "$DESTINATION_DIR/$BINARY_NAME"

  # Verify installation
  if [ -f "$DESTINATION_DIR/$BINARY_NAME" ]; then
    echo "$BINARY_NAME successfully installed in $DESTINATION_DIR"
  else
    echo "Installation failed."
    exit 1
  fi
}

install_service() {
  echo "Installing service..."

cat <<EOF | sudo tee $SERVICE_FILE > /dev/null
[Unit]
Description=FTSOv2 Monitoring Daemon
After=network.target

[Service]
User=$USER
Type=simple
ExecStart=$DESTINATION_DIR/$BINARY_NAME start --rpc-url=http://localhost:9650/ext/bc/C/rpc --block-window 3000 --submit-address= --submit-signature-address= --signing-policy-address=
Restart=never

[Install]
WantedBy=multi-user.target
EOF

  echo "Reloading systemd daemon..."
  sudo systemctl daemon-reload
}

# Main script execution
check_os_type
install_binary
install_service

echo "Installation complete."
echo "You can now edit the service file if needed:"
echo "  sudo vim $SERVICE_FILE"
echo "Then start the service using:"
echo "  sudo systemctl enable --now ftsov2mon.service"
