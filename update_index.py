import re

with open('tavern/internal/www/src/pages/shellv2/index.tsx', 'r') as f:
    content = f.read()

# Replace handleMetaSsh
new_handle = """    const portalIdRef = React.useRef(portalId);
    React.useEffect(() => { portalIdRef.current = portalId; }, [portalId]);

    const handleMetaSsh = React.useCallback((target: string) => {
        if (!portalIdRef.current) {
            return "No active portal connection. Wait for connection or restart shell.";
        }

        const newSessionId = crypto.randomUUID();
        const newTabId = `ssh-${newSessionId}`;
        setTabs(prev => [
            ...prev,
            { id: newTabId, type: "ssh", title: `SSH: ${target}`, target, sessionId: newSessionId }
        ]);
        setActiveTabIndex(tabs.length);
        return null; // Null means success
    }, [tabs.length]);"""

content = re.sub(r'const handleMetaSsh = \(target: string\) => \{.*?\n    \};', new_handle, content, flags=re.DOTALL)

with open('tavern/internal/www/src/pages/shellv2/index.tsx', 'w') as f:
    f.write(content)
