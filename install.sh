#!/bin/bash

LATEST_TAG=$(curl "https://api.github.com/repos/Luganodes/ftsov2mon/tags" | jq -r '.[0].name')
DESTINATION_DIR="/usr/local/bin"
DOWNLOAD_LINK="https://github.com/Luganodes/ftsov2mon/releases/download/$LATEST_TAG/ftsov2mon"
BINARY_NAME="ftsov2mon"
SERVICE_FILE="/etc/systemd/system/ftsov2mon.service"

# Function to check OS type
check_os_type() {
  case "$(uname -s)" in
    Linux*)     OS="Linux" ;;
    Darwin*)    OS="MacOS" ;;
    *)          echo "Unsupported OS"; exit 1 ;;
  esac
}

# Function to install the binary
install_binary() {
  Check if the user is root or has sudo privileges
  if [ "$EUID" -ne 0 ]; then
    echo "Please run this script with sudo or as root."
    exit 1
  fi

  # Check if the destination directory exists
  if [ ! -d "$DESTINATION_DIR" ]; then
    echo "Creating $DESTINATION_DIR..."
    mkdir -p "$DESTINATION_DIR"
  fi

  # Copy the binary to the destination directory
  echo "Installing $BINARY_NAME to $DESTINATION_DIR..."
  wget $DOWNLOAD_LINK -O "$DESTINATION_DIR/$BINARY_NAME"
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

    cat <<EOF > $SERVICE_FILE
    [Unit]
    Description=FTSOv2 Monitoring Daemon
    After=network.target

    [Service]
    User=$USER
    Type=simple
    ExecStart=/usr/local/bin/ftsov2mon start --rpc-url=http://localhost:9650/ext/bc/C/rpc --block-window 3000 --submit-address= --submit-signature-address= --signing-policy-address=
    Restart=never

    [Install]
    WantedBy=multi-user.target
    EOF

    echo "Reloading systemd daemon"
    sudo systemctl daemon-reload
}

# Check the OS type
check_os_type

# Install the binary
install_binary

# Success message
echo "Installation complete."
echo "Please edit the service now and start it"
sudo vim $SERVICE_FILE
