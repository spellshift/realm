import React from 'react';
import { Box, Text, Code } from '@chakra-ui/react';

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
      zIndex={9999}
      pointerEvents="none"
      border="1px solid"
      borderColor="gray.600"
      transform="translateY(10px)" // Add some offset so it doesn't cover the text directly
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
      <Text fontSize="xs" lineHeight="short">
        {description}
      </Text>
    </Box>
  );
};
