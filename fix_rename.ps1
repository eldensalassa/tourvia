$files = Get-ChildItem -Path src -Recurse -Filter *.rs | Where-Object { $_.Name -ne 'theme.rs' }
foreach ($file in $files) {
    $content = Get-Content $file.FullName -Raw
    $content = [regex]::Replace($content, 'theme::([A-Z_]+(?:\(\)[A-Z_]*)+)', {
        param($match)
        $clean = $match.Value -replace '\(\)', ''
        return $clean + '()'
    })
    Set-Content -Path $file.FullName -Value $content -NoNewline
}
