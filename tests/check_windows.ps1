Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class WinTest {
    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    public static void CheckEdge() {
        EnumWindows((hWnd, lParam) => {
            StringBuilder sb = new StringBuilder(512);
            GetWindowText(hWnd, sb, 512);
            string title = sb.ToString();
            if (!string.IsNullOrEmpty(title) && (title.Contains("Edge") || title.Contains("Test") || title.Contains("about"))) {
                Console.WriteLine(hWnd + ": " + title);
            }
            return true;
        }, IntPtr.Zero);
    }
}
"@

$tempProfile = "$env:TEMP\edge_test_profile"
$edge = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
$p = Start-Process $edge -ArgumentList "--user-data-dir=`"$tempProfile`" --new-window about:blank" -PassThru
Start-Sleep -Seconds 3
[WinTest]::CheckEdge()
$p.Kill()
