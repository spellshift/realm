import React, { useRef, useLayoutEffect, useState } from 'react';
import { Box, Code } from '@chakra-ui/react';
import ReactMarkdown from 'react-markdown';

interface DocTooltipProps {
  signature: string;
  description: string;
  x: number;
  y: number;
  visible: boolean;
  onMouseEnter?: () => void;
  onMouseLeave?: () => void;
}

export const DocTooltip: React.FC<DocTooltipProps> = ({ signature, description, x, y, visible, onMouseEnter, onMouseLeave }) => {
  const tooltipRef = useRef<HTMLDivElement>(null);
  const [adjustedPos, setAdjustedPos] = useState({ x, y });

  useLayoutEffect(() => {
    if (visible && tooltipRef.current) {
      const rect = tooltipRef.current.getBoundingClientRect();
      const margin = 10;
      let newX = x;
      let newY = y;

      if (y + rect.height + margin > window.innerHeight) {
        newY = window.innerHeight - rect.height - margin;
      }
      if (x + rect.width + margin > window.innerWidth) {
        newX = window.innerWidth - rect.width - margin;
      }

      setAdjustedPos({ x: newX, y: newY });
    }
  }, [visible, x, y, signature, description]);

  if (!visible) return null;

  return (
    <Box
      ref={tooltipRef}
      position="fixed"
      top={`${adjustedPos.y}px`}
      left={`${adjustedPos.x}px`}
      bg="gray.800"
      color="white"
      p={3}
      borderRadius="md"
      boxShadow="dark-lg"
      maxWidth="400px"
      maxHeight="300px"
      overflowY="auto"
      zIndex={9999}
      pointerEvents="auto"
      border="1px solid"
      borderColor="gray.600"
      transform="translateY(10px)"
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
    >
      <Code
        display="block"
        whiteSpace="pre-wrap"
        bg="blackAlpha.400"
        p={2}
        mb={2}
        borderRadius="sm"
        fontSize="xs"
        fontFamily="monospace"
        color="cyan.300"
      >
        {signature}
      </Code>
      <Box fontSize="xs" lineHeight="short" sx={{
          'p': { marginBottom: '0.5rem' },
          'code': { bg: 'blackAlpha.400', px: 1, borderRadius: 'sm', fontFamily: 'monospace', whiteSpace: 'pre-wrap' },
          'pre': { bg: 'blackAlpha.400', p: 2, borderRadius: 'sm', overflowX: 'auto', marginBottom: '0.5rem', tabSize: 4 },
          'ul': { paddingLeft: '1rem', marginBottom: '0.5rem' },
          'li': { marginBottom: '0.25rem' }
      }}>
        <ReactMarkdown>{description}</ReactMarkdown>
      </Box>
    </Box>
  );
};
