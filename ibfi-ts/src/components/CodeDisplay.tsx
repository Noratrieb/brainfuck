import React from 'react';

interface CodeDisplayProps {
    code: string,
    index: number,
}

const CodeDisplay = ({code, index}: CodeDisplayProps) => {

    const firstCodePart = code.substr(0, index);
    const secondCodePart = code.substr(index + 1, code.length - index + 1);

    return (
        <div className="code-display-wrapper">
            <code>{firstCodePart}</code>
            <code style={{backgroundColor: "red"}}>{code[index] || " "}</code>
            <code>{secondCodePart}</code>
        </div>
    );
};

export default CodeDisplay;