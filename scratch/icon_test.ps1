function Create-CustomMonitorIcon {
    # Render at exact 16x16 for 100% crispness in taskbar tray
    $bmp = New-Object System.Drawing.Bitmap(16, 16, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
    $g.Clear([System.Drawing.Color]::Transparent)

    $whiteBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 255, 255, 255))
    $blueBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 0, 180, 255))
    $goldBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 255, 220, 0))
    $darkBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 30, 30, 35))

    # 1. Stand Base (Rows 13-14, Columns 3-12)
    $g.FillRectangle($whiteBrush, 3, 13, 10, 2)

    # 2. Stand Neck (Rows 10-12, Columns 6-9)
    $g.FillRectangle($whiteBrush, 6, 10, 4, 3)

    # 3. Outer Monitor Bezel (Rows 0-9, Columns 0-15)
    $g.FillRectangle($whiteBrush, 0, 0, 16, 10)

    # 4. Display Screen (Rows 1-8, Columns 1-14)
    $g.FillRectangle($blueBrush, 1, 1, 14, 8)

    # 5. Bright Sun in Center (Rows 3-6, Columns 6-9)
    $g.FillRectangle($goldBrush, 6, 3, 4, 4)

    # Clean up
    $g.Dispose()
    $whiteBrush.Dispose()
    $blueBrush.Dispose()
    $goldBrush.Dispose()
    $darkBrush.Dispose()

    $hIcon = $bmp.GetHicon()
    return [System.Drawing.Icon]::FromHandle($hIcon)
}
