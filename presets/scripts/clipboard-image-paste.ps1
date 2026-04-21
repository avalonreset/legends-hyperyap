Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$img = $null
$deadline = (Get-Date).AddMilliseconds(5000)

while ((Get-Date) -lt $deadline -and -not $img) {
    try {
        $img = [System.Windows.Forms.Clipboard]::GetImage()
    } catch {
        $img = $null
    }

    if (-not $img) {
        Start-Sleep -Milliseconds 50
    }
}

if (-not $img) {
    exit 1
}

try {
    $dir = "$env:USERPROFILE\screenshots"
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir | Out-Null }
    $timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss_fff"
    $path = "$dir\screenshot_$timestamp.png"
    $img.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    [System.Windows.Forms.Clipboard]::SetText($path.Replace('\', '/'))
    exit 0
} catch {
    exit 1
} finally {
    $img.Dispose()
}
