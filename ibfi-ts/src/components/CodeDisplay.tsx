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
            <span>{firstCodePart}</span>
            <span style={{backgroundColor: "red"}}>{code[index] || " "}</span>
            <span>{secondCodePart}</span>
        </div>
    );
};

export default CodeDisplay;