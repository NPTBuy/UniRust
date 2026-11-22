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
            if (!string.IsNullOrEmpty(title)) {
                Console.WriteLine(hWnd + ": " + title);
            }
            return true;
        }, IntPtr.Zero);
    }
}
"@

$listener = New-Object System.Net.HttpListener
$listener.Prefixes.Add("http://127.0.0.1:8989/")
$listener.Start()

$htmlFile = "d:\UniRush\tests\e2e_typing_test.html"
$htmlContent = [System.IO.File]::ReadAllBytes($htmlFile)

$task = [System.Threading.Tasks.Task]::Run([Action]{
    while ($listener.IsListening) {
        try {
            $ctx = $listener.GetContext()
            $ctx.Response.ContentType = "text/html; charset=utf-8"
            $ctx.Response.OutputStream.Write($htmlContent, 0, $htmlContent.Length)
            $ctx.Response.Close()
        } catch { break }
    }
})

$edge = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
$tempProfile = "$env:TEMP\edge_test_profile_diag"
$p = Start-Process $edge -ArgumentList "--user-data-dir=`"$tempProfile`" --no-first-run --no-default-browser-check --app=http://127.0.0.1:8989/" -PassThru

Start-Sleep -Seconds 6
[WinTest]::CheckEdge()

$p.Kill()
$listener.Stop()
