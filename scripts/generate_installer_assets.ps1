Add-Type -AssemblyName System.Drawing

$installerDir = "F:\league app\installer"
if (-not (Test-Path $installerDir)) {
    New-Item -ItemType Directory -Path $installerDir -Force | Out-Null
}

# 1. Create Sidebar Bitmap (164 x 314)
$bmpSide = New-Object System.Drawing.Bitmap 164, 314
$g = [System.Drawing.Graphics]::FromImage($bmpSide)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::ClearTypeGridFit

# Gradient Background
$rectSide = New-Object System.Drawing.Rectangle 0, 0, 164, 314
$brushBg = New-Object System.Drawing.Drawing2D.LinearGradientBrush $rectSide, ([System.Drawing.Color]::FromArgb(6, 11, 20)), ([System.Drawing.Color]::FromArgb(12, 22, 38)), [System.Drawing.Drawing2D.LinearGradientMode]::Vertical
$g.FillRectangle($brushBg, $rectSide)

# Cyan Accent Bar
$penCyan = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(0, 200, 255)), 2
$g.DrawLine($penCyan, 162, 0, 162, 314)

# Top Glow
$glowBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(35, 0, 200, 255))
$g.FillEllipse($glowBrush, -20, -20, 140, 140)

# Hexagon Logo
$logoBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(0, 210, 255))
$pts = @(
    (New-Object System.Drawing.Point 40, 45),
    (New-Object System.Drawing.Point 65, 30),
    (New-Object System.Drawing.Point 90, 45),
    (New-Object System.Drawing.Point 90, 75),
    (New-Object System.Drawing.Point 65, 90),
    (New-Object System.Drawing.Point 40, 75)
)
$g.DrawPolygon((New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(0, 210, 255)), 3), $pts)
$g.FillPolygon((New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(50, 0, 210, 255))), $pts)

# Inner Core
$innerPts = @(
    (New-Object System.Drawing.Point 52, 53),
    (New-Object System.Drawing.Point 65, 45),
    (New-Object System.Drawing.Point 78, 53),
    (New-Object System.Drawing.Point 78, 67),
    (New-Object System.Drawing.Point 65, 75),
    (New-Object System.Drawing.Point 52, 67)
)
$g.FillPolygon($logoBrush, $innerPts)

# Typography
$fontTitle = New-Object System.Drawing.Font 'Segoe UI', 12, [System.Drawing.FontStyle]::Bold
$fontSub = New-Object System.Drawing.Font 'Segoe UI', 8, [System.Drawing.FontStyle]::Bold
$fontBody = New-Object System.Drawing.Font 'Segoe UI', 7.5, [System.Drawing.FontStyle]::Regular
$brushWhite = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(245, 250, 255))
$brushCyan = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(0, 210, 255))
$brushDim = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(148, 163, 184))

$g.DrawString('RIFTCORE', $fontTitle, $brushWhite, 28, 105)
$g.DrawString('COMPANION PRO', $fontSub, $brushCyan, 28, 128)

# Features list
$g.DrawString('• In-Game Tactical Overlay', $fontBody, $brushDim, 14, 165)
$g.DrawString('• Live Matchup Tips', $fontBody, $brushDim, 14, 185)
$g.DrawString('• Next Skill & Item HUD', $fontBody, $brushDim, 14, 205)
$g.DrawString('• Cooldown & CS Timers', $fontBody, $brushDim, 14, 225)
$g.DrawString('• Auto-Accept Matches', $fontBody, $brushDim, 14, 245)

$g.DrawString('v1.1.0 Stable Release', $fontSub, $brushCyan, 24, 285)

$sidePath = Join-Path $installerDir "installerSidebar.bmp"
$bmpSide.Save($sidePath, [System.Drawing.Imaging.ImageFormat]::Bmp)
$g.Dispose()
$bmpSide.Dispose()

# 2. Create Header Bitmap (150 x 57)
$bmpHead = New-Object System.Drawing.Bitmap 150, 57
$gh = [System.Drawing.Graphics]::FromImage($bmpHead)
$gh.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$rectHead = New-Object System.Drawing.Rectangle 0, 0, 150, 57
$brushHeadBg = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(8, 13, 24))
$gh.FillRectangle($brushHeadBg, $rectHead)

# Mini Logo on Right
$headPts = @(
    (New-Object System.Drawing.Point 105, 18),
    (New-Object System.Drawing.Point 120, 9),
    (New-Object System.Drawing.Point 135, 18),
    (New-Object System.Drawing.Point 135, 36),
    (New-Object System.Drawing.Point 120, 45),
    (New-Object System.Drawing.Point 105, 36)
)
$gh.DrawPolygon((New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(0, 210, 255)), 2), $headPts)
$gh.FillPolygon((New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(40, 0, 210, 255))), $headPts)

$headPath = Join-Path $installerDir "installerHeader.bmp"
$bmpHead.Save($headPath, [System.Drawing.Imaging.ImageFormat]::Bmp)
$gh.Dispose()
$bmpHead.Dispose()

Write-Host "Installer assets generated in $installerDir"
