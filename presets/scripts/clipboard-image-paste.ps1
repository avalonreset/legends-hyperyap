Add-Type -AssemblyName System.Windows.Forms

$img = [System.Windows.Forms.Clipboard]::GetImage()

if ($img) {
    $dir = "$env:USERPROFILE\screenshots"
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir | Out-Null }
    $timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
    $path = "$dir\screenshot_$timestamp.png"
    $img.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $img.Dispose()
    [System.Windows.Forms.Clipboard]::SetText($path.Replace('\', '/'))
}
