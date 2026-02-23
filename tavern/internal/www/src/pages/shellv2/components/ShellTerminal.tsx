import React, { forwardRef } from "react";

const ShellTerminal = forwardRef<HTMLDivElement>((props, ref) => {
    return (
        <div ref={ref} style={{ height: "100%", width: "100%" }} />
    );
});

export default ShellTerminal;
