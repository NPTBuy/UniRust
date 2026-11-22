Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class WinDiag {
    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();

    [DllImport("user32.dll")]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool BringWindowToTop(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    [DllImport("user32.dll")]
    public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    public static IntPtr FindEdge() {
        IntPtr result = IntPtr.Zero;
        EnumWindows((hWnd, lParam) => {
            StringBuilder sb = new StringBuilder(512);
            GetWindowText(hWnd, sb, 512);
            string title = sb.ToString();
            if (title.Contains("127.0.0.1") || title.Contains("Edge") || title.Contains("Typing")) {
                result = hWnd;
                return false;
            }
            return true;
        }, IntPtr.Zero);
        return result;
    }

    public static void BringToFront(IntPtr hWnd) {
        keybd_event(0x12, 0, 0, UIntPtr.Zero);
        keybd_event(0x12, 0, 2, UIntPtr.Zero);
        ShowWindow(hWnd, 9);
        BringWindowToTop(hWnd);
        SetForegroundWindow(hWnd);
    }

    public static void ShowFg() {
        IntPtr fg = GetForegroundWindow();
        StringBuilder sb = new StringBuilder(256);
        GetWindowText(fg, sb, 256);
        Console.WriteLine("Active HWND: " + fg + ", Title: " + sb.ToString());
    }
}
"@

$edge = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
$tempProfile = "$env:TEMP\edge_diag_profile"
$p = Start-Process $edge -ArgumentList "--user-data-dir=`"$tempProfile`" --no-first-run --no-default-browser-check --app=http://127.0.0.1:8989/" -PassThru
Start-Sleep -Seconds 3

$hwnd = [WinDiag]::FindEdge()
[Console]::WriteLine("Found Edge HWND: " + $hwnd)
if ($hwnd -ne [IntPtr]::Zero) {
    [WinDiag]::BringToFront($hwnd)
    Start-Sleep -Milliseconds 500
    [WinDiag]::ShowFg()
}

$p.Kill()
