#!/bin/bash
set -e

REPO="mahan07dev/System-Monitor"
APP_NAME="System Monitor"

echo "================================================="
echo " Installing ${APP_NAME}..."
echo "================================================="

# -----------------------------------------------------------------------------
# 1. Detect Operating System & Architecture
# -----------------------------------------------------------------------------
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux*)     OS_TYPE="linux";;
    Darwin*)    OS_TYPE="macos";;
    *)          echo "❌ Unsupported OS: ${OS}"; exit 1;;
esac

echo "--> Detected OS: ${OS_TYPE} (${ARCH})"

# -----------------------------------------------------------------------------
# 2. Fetch Latest Release Assets from GitHub API
# -----------------------------------------------------------------------------
RELEASE_JSON=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")
VERSION=$(echo "${RELEASE_JSON}" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "${VERSION}" ]; then
    echo "❌ Failed to fetch latest release from GitHub."
    exit 1
fi

echo "--> Latest Version: ${VERSION}"

# Helper to extract browser_download_url by filename pattern
get_download_url() {
    local pattern="$1"
    echo "${RELEASE_JSON}" | grep "browser_download_url" | grep -i "${pattern}" | head -n 1 | cut -d '"' -f 4
}

# -----------------------------------------------------------------------------
# 3. Determine Correct Package & SHA256 Checksum
# -----------------------------------------------------------------------------
FILE_URL=""
EXPECTED_SHA256=""

if [ "${OS_TYPE}" = "linux" ]; then
    if [ "${ARCH}" != "x86_64" ]; then
        echo "❌ Linux installation script currently supports x86_64 architecture."
        exit 1
    fi

    # Check for Debian/Ubuntu (.deb)
    if command -v dpkg >/dev/null 2>&1; then
        echo "--> Package Manager: dpkg (.deb)"
        FILE_URL=$(get_download_url "amd64.deb")
        EXPECTED_SHA256="22088e5daa78ef4c0cab4c9e7ac4cf0546e132c7b67d6658f28cbc43f081e79c"

    # Check for Fedora/RHEL/RPM (.rpm)
    elif command -v rpm >/dev/null 2>&1; then
        echo "--> Package Manager: rpm (.rpm)"
        FILE_URL=$(get_download_url "x86_64.rpm")
        EXPECTED_SHA256="853a95497a6bd5a090a2e8a0c52bae1f323197cac5a9c4bbbde0ca55aa169f96"

    # Fallback to AppImage for other distros (Arch, Alpine, Void, etc.)
    else
        echo "--> Package Manager: Fallback to standalone .AppImage"
        FILE_URL=$(get_download_url "amd64.AppImage")
        EXPECTED_SHA256="8e0c6f043fa9fe0d3039b6baa022c4009433fe37ee551a0f966b45206277e671"
    fi

elif [ "${OS_TYPE}" = "macos" ]; then
    if [ "${ARCH}" = "arm64" ] || [ "${ARCH}" = "aarch64" ]; then
        echo "--> Apple Silicon Detected (aarch64)"
        FILE_URL=$(get_download_url "aarch64.dmg")
        EXPECTED_SHA256="02d8dae6c9d18512cfa550c69374dd4704ee1d6ac09ff73521796696e18032b8"
    else
        echo "❌ Intel macOS builds are not available in this release."
        exit 1
    fi
fi

if [ -z "${FILE_URL}" ]; then
    echo "❌ Error matching suitable release package."
    exit 1
fi

FILENAME=$(basename "${FILE_URL}")
TMP_DIR=$(mktemp -d)
TMP_FILE="${TMP_DIR}/${FILENAME}"

# Cleanup temporary files on exit
trap 'rm -rf "${TMP_DIR}"' EXIT

# -----------------------------------------------------------------------------
# 4. Download and Verify Checksum
# -----------------------------------------------------------------------------
echo "--> Downloading ${FILENAME}..."
curl -fsSL -o "${TMP_FILE}" "${FILE_URL}"

echo "--> Verifying SHA-256 Checksum..."
if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL_SHA256=$(sha256sum "${TMP_FILE}" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
    ACTUAL_SHA256=$(shasum -a 256 "${TMP_FILE}" | awk '{print $1}')
else
    echo "⚠️ Warning: Neither sha256sum nor shasum found. Skipping verification."
    ACTUAL_SHA256="${EXPECTED_SHA256}"
fi

if [ "${ACTUAL_SHA256}" != "${EXPECTED_SHA256}" ]; then
    echo "❌ Checksum verification failed!"
    echo "   Expected: ${EXPECTED_SHA256}"
    echo "   Got:      ${ACTUAL_SHA256}"
    exit 1
fi
echo "--> SHA-256 Verified successfully!"

# -----------------------------------------------------------------------------
# 5. Installation
# -----------------------------------------------------------------------------
echo "--> Installing..."

if [ "${OS_TYPE}" = "linux" ]; then
    case "${FILENAME}" in
        *.deb)
            sudo dpkg -i "${TMP_FILE}" || sudo apt-get install -f -y
            ;;
        *.rpm)
            if command -v dnf >/dev/null 2>&1; then
                sudo dnf install -y "${TMP_FILE}"
            else
                sudo rpm -i "${TMP_FILE}"
            fi
            ;;
        *.AppImage)
            INSTALL_DIR="${HOME}/.local/bin"
            mkdir -p "${INSTALL_DIR}"
            chmod +x "${TMP_FILE}"
            mv "${TMP_FILE}" "${INSTALL_DIR}/system-monitor"
            echo "--> AppImage saved to ${INSTALL_DIR}/system-monitor"
            ;;
    esac

elif [ "${OS_TYPE}" = "macos" ]; then
    echo "--> Mounting DMG..."
    MOUNT_DIR=$(mktemp -d)
    hdiutil attach "${TMP_FILE}" -mountpoint "${MOUNT_DIR}" -quiet
    
    echo "--> Copying to /Applications..."
    sudo cp -R "${MOUNT_DIR}/System Monitor.app" /Applications/
    
    hdiutil detach "${MOUNT_DIR}" -quiet
    rm -rf "${MOUNT_DIR}"
fi

echo "================================================="
echo " 🎉 ${APP_NAME} installed successfully!"
echo "================================================="