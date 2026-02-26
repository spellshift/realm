import React from 'react';
import { Box, Code } from '@chakra-ui/react';
import ReactMarkdown from 'react-markdown';

interface DocTooltipProps {
  signature: string;
  description: string;
  x: number;
  y: number;
  visible: boolean;
}

export const DocTooltip: React.FC<DocTooltipProps> = ({ signature, description, x, y, visible }) => {
  if (!visible) return null;

  return (
    <Box
      position="fixed"
      top={`${y}px`}
      left={`${x}px`}
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
          'code': { bg: 'blackAlpha.400', px: 1, borderRadius: 'sm', fontFamily: 'monospace' },
          'pre': { bg: 'blackAlpha.400', p: 2, borderRadius: 'sm', overflowX: 'auto', marginBottom: '0.5rem' },
          'ul': { paddingLeft: '1rem', marginBottom: '0.5rem' },
          'li': { marginBottom: '0.25rem' }
      }}>
        <ReactMarkdown>{description}</ReactMarkdown>
      </Box>
    </Box>
  );
};
