$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$assetDir = Join-Path $PSScriptRoot '..\riftcore-native\assets\tft_icons'
$assetDir = [System.IO.Path]::GetFullPath($assetDir)
Get-ChildItem -LiteralPath $assetDir -Filter '*.optimized' -File | ForEach-Object { [System.IO.File]::Delete($_.FullName) }
$files = Get-ChildItem -LiteralPath $assetDir -Filter '*.png' -File

foreach ($file in $files) {
    $source = [System.Drawing.Image]::FromFile($file.FullName)
    try {
        $side = [Math]::Min($source.Width, $source.Height)
        $crop = if ($file.BaseName.StartsWith('champion_', [StringComparison]::OrdinalIgnoreCase)) {
            [System.Drawing.Rectangle]::new([int](($source.Width - $side) / 2), [int](($source.Height - $side) / 2), $side, $side)
        } else {
            [System.Drawing.Rectangle]::new(0, 0, $source.Width, $source.Height)
        }

        $bitmap = [System.Drawing.Bitmap]::new(64, 64, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
        try {
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            try {
                $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
                $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
                $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
                $graphics.DrawImage($source, [System.Drawing.Rectangle]::new(0, 0, 64, 64), $crop, [System.Drawing.GraphicsUnit]::Pixel)
            } finally {
                $graphics.Dispose()
            }
            $temporary = "$($file.FullName).$([Guid]::NewGuid().ToString('N')).optimized"
            $bitmap.Save($temporary, [System.Drawing.Imaging.ImageFormat]::Png)
            $source.Dispose()
            $source = $null
            Move-Item -LiteralPath $temporary -Destination $file.FullName -Force
        } finally {
            $bitmap.Dispose()
        }
    } finally {
        if ($null -ne $source) { $source.Dispose() }
    }
}

Write-Output "Optimized $($files.Count) TFT icons to 64×64 PNGs."
