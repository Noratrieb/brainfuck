import React from 'react';

interface ProgramOutputProps {
    text: string
}

const ProgramOutput = ({text}: ProgramOutputProps) => {
    return (
        <div className="bf-output">
            <textarea readOnly className="output-area" value={text}/>
        </div>
    );
};

export default ProgramOutput;