/**
 * * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

module.exports = {
  docs: [
  {
    type: 'doc',
    id: 'welcome',
  },
  {
    type: 'doc',
    id: 'overview',
  },
  {
    type: 'doc',
    id: 'getting_started',
  },
  {
    type: 'category',
    label: 'Libraries',
    collapsed: false,
    items: [
    {
      type: 'doc',
      id: 'libraries/overview',
      label: 'Overview',
    },
    {
      type: 'category',
      label: 'Rust',
      items: [
        {
          type: 'doc',
          id: 'libraries/rust/getting_started',
          label: 'Getting Started',
        },
        {
          type: 'category',
          label: 'How to',
          items: [
            {
              type: 'doc',
              id: 'libraries/rust/how_to/get_node_info',
              label: 'Get Node Info',
            },
            {
              type: 'doc',
              id: 'libraries/rust/how_to/get_block',
              label: 'Get A Block',
            },
            // {
            //   type: 'doc',
            //   id: 'libraries/rust/how_to/create_block',
            //   label: 'Create A Block',
            // },
            {
              type: 'doc',
              id: 'libraries/rust/how_to/post_block',
              label: 'Post A Block',
            },
            {
              type: 'doc',
              id: 'libraries/rust/how_to/generate_mnemonic',
              label: 'Generate A Mnemonic',
            },
            {
              type: 'doc',
              id: 'libraries/rust/how_to/generate_addresses',
              label: 'Generate Addresses',
            },
            // {
            //   type: 'doc',
            //   id: 'libraries/rust/how_to/get_output',
            //   label: 'Get An Output',
            // },
            // {
            //   type: 'doc',
            //   id: 'libraries/rust/how_to/build_output',
            //   label: 'Build An Output',
            // },
            // {
            //   type: 'doc',
            //   id: 'libraries/rust/how_to/prepare_sign_transaction',
            //   label: 'Prepare And Sign A Transaction',
            // },
          ]
        },
        {
          type: 'doc',
          id: 'libraries/rust/api_reference',
          label: 'API Reference'
        },
      ]
    },
    {
      type: 'category',
      label: 'Node.js',
      items: [
        {
          type: 'doc',
          id: 'libraries/nodejs/getting_started',
          label: 'Getting Started'
        },
        {
          type: 'doc',
          id: 'libraries/nodejs/examples',
          label: 'Examples'
        },
        {
          type: 'doc',
          id: 'libraries/nodejs/api_reference',
          label: 'API Reference'
        },
      ]
    },
    {
      type: 'category',
      label: 'Python',
      items: [
        {
          type: 'doc',
          id: 'libraries/python/getting_started',
          label: 'Getting Started'
        },
        {
          type: 'doc',
          id: 'libraries/python/api_reference',
          label: 'API Reference'
        },
      ]
    },
    {
      type: 'category',
      label: 'Java',
      items: [
        {
          type: 'doc',
          id: 'libraries/java/getting_started',
          label: 'Getting Started'
        },
        {
          type: 'doc',
          id: 'libraries/java/how_to/build_output',
          label: 'Build An Output',
        },
        {
          type: 'doc',
          id: 'libraries/java/how_to/create_block',
          label: 'Create A Block',
        },
        {
          type: 'doc',
          id: 'libraries/java/how_to/generate_addresses',
          label: 'Generate Addresses',
        },
        {
          type: 'doc',
          id: 'libraries/java/how_to/generate_mnemonic',
          label: 'Generate A Mnemonic',
        },
        {
          type: 'doc',
          id: 'libraries/java/how_to/get_block',
          label: 'Get A Block',
        },
        {
          type: 'doc',
          id: 'libraries/java/how_to/get_node_info',
          label: 'Get Node Info',
        },
        {
          type: 'doc',
          id: 'libraries/java/how_to/get_output',
          label: 'Get A Output',
        },
        {
          type: 'doc',
          id: 'libraries/java/how_to/post_block',
          label: 'Post A Block',
        },
        {
          type: 'doc',
          id: 'libraries/java/how_to/prepare_sign_transaction',
          label: 'Prepare And Sign A Transaction',
        },
        {
          type: 'doc',
          id: 'libraries/java/api_reference',
          label: 'API Reference'
        },
      ]
    }
    ]
  },
  {
    type: 'doc',
    id: 'troubleshooting',
    label: 'Troubleshooting'
  },
  {
    type: 'doc',
    id: 'contribute',
    label: 'Contribute',
  }
  ]
};
