<#
.SYNOPSIS
  Automated End-to-End Typing-Test Harness for UniRush / UniKey-rs.
  Verifies that normal inputs dispatch atomic backspace + unicode batches with ZERO mid-word Space events.
#>

param(
    [string]$BinaryPath = "d:\REA\target\release\unirust.exe",
    [int]$Port = 8989
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   UniRush / UniKey-rs Fast-Typing E2E Automated Test    " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Stop any previously running unirust or unikey-rs instances
Get-Process -Name "unirust", "unikey-rs" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 500

# 2. Check binary exists
if (-not (Test-Path $BinaryPath)) {
    Write-Error "Binary not found at $BinaryPath. Please run cargo build --release first."
    exit 1
}

# 3. Start unirust.exe in background
Write-Host "[+] Launching $BinaryPath..." -ForegroundColor Yellow
$appProc = Start-Process -FilePath $BinaryPath -PassThru
Start-Sleep -Milliseconds 800

# 4. Setup local lightweight TCP server to host tests/e2e_typing_test.html and receive results
$tcpListener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, $Port)
$tcpListener.Start()
Write-Host "[+] Local test server listening on http://127.0.0.1:$Port/" -ForegroundColor Green

$htmlFile = Join-Path $PSScriptRoot "e2e_typing_test.html"
$htmlContent = [System.IO.File]::ReadAllBytes($htmlFile)

$script:testResult = $null

# Background task to handle HTTP requests over TCP
$serverTask = [System.Threading.Tasks.Task]::Run([Action]{
    while ($true) {
        try {
            $client = $tcpListener.AcceptTcpClient()
            $stream = $client.GetStream()
            $reader = New-Object System.IO.StreamReader($stream, [System.Text.Encoding]::UTF8)
            $writer = New-Object System.IO.StreamWriter($stream, [System.Text.Encoding]::UTF8)
            $writer.AutoFlush = $true

            $line = $reader.ReadLine()
            if ([string]::IsNullOrEmpty($line)) {
                $client.Close()
                continue
            }

            $parts = $line.Split(" ")
            $method = $parts[0]
            $url = if ($parts.Length -gt 1) { $parts[1] } else { "/" }

            # Read headers
            $contentLength = 0
            while ($true) {
                $header = $reader.ReadLine()
                if ([string]::IsNullOrEmpty($header)) { break }
                if ($header.ToLower().StartsWith("content-length:")) {
                    $contentLength = [int]($header.Split(":")[1].Trim())
                }
            }

            if ($method -eq "GET") {
                $headerStr = "HTTP/1.1 200 OK`r`nContent-Type: text/html; charset=utf-8`r`nContent-Length: $($htmlContent.Length)`r`nConnection: close`r`n`r`n"
                $headerBytes = [System.Text.Encoding]::UTF8.GetBytes($headerStr)
                $stream.Write($headerBytes, 0, $headerBytes.Length)
                $stream.Write($htmlContent, 0, $htmlContent.Length)
                $stream.Flush()
                $client.Close()
            }
            elseif ($method -eq "POST" -and $url -eq "/result") {
                $bodyChars = New-Object char[] $contentLength
                $reader.ReadBlock($bodyChars, 0, $contentLength) | Out-Null
                $body = New-Object string($bodyChars, 0, $contentLength)

                $script:testResult = $body

                $ack = '{"status":"ok"}'
                $ackBytes = [System.Text.Encoding]::UTF8.GetBytes($ack)
                $headerStr = "HTTP/1.1 200 OK`r`nContent-Type: application/json`r`nContent-Length: $($ackBytes.Length)`r`nConnection: close`r`n`r`n"
                $headerBytes = [System.Text.Encoding]::UTF8.GetBytes($headerStr)
                $stream.Write($headerBytes, 0, $headerBytes.Length)
                $stream.Write($ackBytes, 0, $ackBytes.Length)
                $stream.Flush()
                $client.Close()
                break
            }
            else {
                $client.Close()
            }
        }
        catch {
            break
        }
    }
})

# 5. Launch Microsoft Edge in isolated app window
$edgePath = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
if (-not (Test-Path $edgePath)) {
    $edgePath = (Get-Command msedge.exe -ErrorAction SilentlyContinue).Source
}

$tempProfile = "$env:TEMP\edge_typing_profile_$(Get-Random)"
Write-Host "[+] Opening Edge browser at http://127.0.0.1:$Port/ ..." -ForegroundColor Yellow
$browserArgs = "--user-data-dir=`"$tempProfile`" --no-first-run --no-default-browser-check --app=http://127.0.0.1:$Port/"
$browserProc = Start-Process -FilePath $edgePath -ArgumentList $browserArgs -PassThru

# 6. Bring browser window to foreground and send real OS keystrokes
Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;

public class NativeInput {
    [DllImport("user32.dll")]
    public static extern uint SendInput(uint nInputs, INPUT[] pInputs, int cbSize);

    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder text, int count);

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc enumProc, IntPtr lParam);
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [StructLayout(LayoutKind.Sequential)]
    public struct KEYBDINPUT {
        public ushort wVk;
        public ushort wScan;
        public uint dwFlags;
        public uint time;
        public UIntPtr dwExtraInfo;
    }

    [StructLayout(LayoutKind.Explicit, Size = 40)]
    public struct INPUT {
        [FieldOffset(0)] public uint type;
        [FieldOffset(8)] public KEYBDINPUT ki;
    }

    public static IntPtr FindWindowByTitleSubstring() {
        IntPtr result = IntPtr.Zero;
        EnumWindows((hWnd, lParam) => {
            StringBuilder sb = new StringBuilder(512);
            GetWindowText(hWnd, sb, 512);
            string title = sb.ToString();
            if (title.IndexOf("127.0.0.1", StringComparison.OrdinalIgnoreCase) >= 0
                || title.IndexOf("Typing", StringComparison.OrdinalIgnoreCase) >= 0
                || title.IndexOf("TEST_", StringComparison.OrdinalIgnoreCase) >= 0) {
                result = hWnd;
                return false;
            }
            return true;
        }, IntPtr.Zero);
        return result;
    }

    public static string GetTitle(IntPtr hWnd) {
        StringBuilder sb = new StringBuilder(512);
        GetWindowText(hWnd, sb, 512);
        return sb.ToString();
    }

    public static void SendKey(ushort vk, int delayMs = 35) {
        INPUT[] down = new INPUT[1];
        down[0].type = 1;
        down[0].ki.wVk = vk;
        down[0].ki.dwFlags = 0;
        down[0].ki.dwExtraInfo = UIntPtr.Zero; // Real physical input (no UNIK tag)
        SendInput(1, down, 40);

        System.Threading.Thread.Sleep(12);

        INPUT[] up = new INPUT[1];
        up[0].type = 1;
        up[0].ki.wVk = vk;
        up[0].ki.dwFlags = 2; // KEYEVENTF_KEYUP
        up[0].ki.dwExtraInfo = UIntPtr.Zero;
        SendInput(1, up, 40);

        System.Threading.Thread.Sleep(delayMs);
    }

    public static void TypeText(string text) {
        foreach (char ch in text) {
            ushort vk = 0;
            if (ch >= 'a' && ch <= 'z') {
                vk = (ushort)(ch - 'a' + 0x41);
            } else if (ch >= 'A' && ch <= 'Z') {
                vk = (ushort)(ch - 'A' + 0x41);
            } else if (ch == ' ') {
                vk = 0x20;
            } else if (ch == '\b') {
                vk = 0x08;
            }
            if (vk != 0) {
                SendKey(vk, 35);
            }
        }
    }
}
"@

# Locate Edge window
Write-Host "[+] Waiting for test page window to appear..." -ForegroundColor Cyan
$targetHwnd = [IntPtr]::Zero
for ($i = 0; $i -lt 30; $i++) {
    Start-Sleep -Milliseconds 300
    $targetHwnd = [NativeInput]::FindWindowByTitleSubstring()
    if ($targetHwnd -ne [IntPtr]::Zero) {
        break
    }
}

if ($targetHwnd -ne [IntPtr]::Zero) {
    Write-Host "[+] Found test window (HWND: $targetHwnd). Bringing to foreground..." -ForegroundColor Green
    [NativeInput]::SetForegroundWindow($targetHwnd) | Out-Null
    Start-Sleep -Milliseconds 1200
} else {
    Write-Host "[!] Could not locate window with target title, continuing..." -ForegroundColor Yellow
}

Write-Host "[+] Injecting real OS keystrokes for 9 target words..." -ForegroundColor Cyan

# Word 1: "tieengs " -> "tiếng "
Write-Host "  -> Typing 'tieengs ' [target: tiếng]"
[NativeInput]::TypeText("tieengs ")

# Word 2: "vieetj " -> "việt "
Write-Host "  -> Typing 'vieetj ' [target: việt]"
[NativeInput]::TypeText("vieetj ")

# Word 3: "truongwf " -> "trường "
Write-Host "  -> Typing 'truongwf ' [target: trường]"
[NativeInput]::TypeText("truongwf ")

# Word 4: "nguoiwf " -> "người "
Write-Host "  -> Typing 'nguoiwf ' [target: người]"
[NativeInput]::TypeText("nguoiwf ")

# Word 5: "suawr " -> "sửa "
Write-Host "  -> Typing 'suawr ' [target: sửa]"
[NativeInput]::TypeText("suawr ")

# Word 6: "giuawx " -> "giữa "
Write-Host "  -> Typing 'giuawx ' [target: giữa]"
[NativeInput]::TypeText("giuawx ")

# Word 7: "ddesn" + Backspace + "en " -> "đến " (mid-word repair)
Write-Host "  -> Typing 'ddesn' + Backspace + 'en ' [target: đến]"
[NativeInput]::TypeText("ddesn")
[NativeInput]::SendKey(0x08, 50) # VK_BACK
[NativeInput]::TypeText("en ")

# Word 8: "hoaf " -> "hoà "
Write-Host "  -> Typing 'hoaf ' [target: hoà]"
[NativeInput]::TypeText("hoaf ")

# Word 9: "thuyr " -> "thuỷ "
Write-Host "  -> Typing 'thuyr ' [target: thuỷ]"
[NativeInput]::TypeText("thuyr ")

Write-Host "[+] Keystroke sequence finished. Waiting for test result..." -ForegroundColor Yellow

# Wait up to 10 seconds for result from HTTP or window title
$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
while ($null -eq $script:testResult -and $stopwatch.ElapsedMilliseconds -lt 10000) {
    Start-Sleep -Milliseconds 200

    if ($targetHwnd -ne [IntPtr]::Zero) {
        $curTitle = [NativeInput]::GetTitle($targetHwnd)
        if ($curTitle.StartsWith("TEST_PASSED:")) {
            $words = $curTitle.Substring(12).Trim().Split(",") | ForEach-Object { $_.Trim() }
            $script:testResult = '{"status":"PASSED","passed":true,"completedWords":' + (ConvertTo-Json -Compress $words) + '}'
            break
        } elseif ($curTitle.StartsWith("TEST_FAILED:")) {
            $err = $curTitle.Substring(12).Trim()
            $script:testResult = '{"status":"FAILED","passed":false,"error":"' + $err + '"}'
            break
        }
    }
}

# Cleanup TCP listener
try {
    $tcpListener.Stop()
} catch {}

# Cleanup Edge process
try {
    $browserProc.Kill()
} catch {}

# Cleanup unirust process
try {
    $appProc.Kill()
} catch {}

# Cleanup temp profile
try {
    Remove-Item -Path $tempProfile -Recurse -Force -ErrorAction SilentlyContinue
} catch {}

# 7. Evaluate test result
if ($null -eq $script:testResult) {
    Write-Host "[FAIL] E2E test timed out without receiving a result!" -ForegroundColor Red
    exit 1
}

Write-Host "[*] Received result payload:" -ForegroundColor Gray
Write-Host $script:testResult -ForegroundColor Gray

$json = $script:testResult | ConvertFrom-Json

if ($json.passed -eq $true) {
    Write-Host ""
    Write-Host "==========================================================" -ForegroundColor Green
    Write-Host " [PASS] E2E Vietnamese Typing Test 100% SUCCEEDED!       " -ForegroundColor Green
    Write-Host " Words Completed: $($json.completedWords -join ', ')" -ForegroundColor Green
    Write-Host " Zero premature mid-word spaces detected.                 " -ForegroundColor Green
    Write-Host " Atomic SendInput batches verified cleanly.               " -ForegroundColor Green
    Write-Host "==========================================================" -ForegroundColor Green
    exit 0
} else {
    Write-Host ""
    Write-Host "==========================================================" -ForegroundColor Red
    Write-Host " [FAIL] E2E Vietnamese Typing Test FAILED!               " -ForegroundColor Red
    Write-Host " Error: $($json.error)                                    " -ForegroundColor Red
    Write-Host "==========================================================" -ForegroundColor Red
    exit 1
}
