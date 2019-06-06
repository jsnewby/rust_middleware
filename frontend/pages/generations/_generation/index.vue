<template>
  <div class="app-generation-details">
    <PageHeader
      title="Generation Details"
      :has-crumbs="true"
      :has-nav="true"
      :page="{to: '/generations', name: 'Generations'}"
      :subpage="{to: `/generations/${$route.params.generation}`, name: 'Generation Details'}"
      :prev="prev"
      :next="next"
    />
    <GenerationDetails
      :data="generation"
      :dynamic-data="height"
    />
    <MicroBlocks>
      <MicroBlock
        v-for="(microBlock, number) in generation.micro_blocks"
        :key="number"
        :data="microBlock"
      >
        <TXListItem
          v-for="(transaction, index) in microBlock.transactions"
          :key="index"
          :data="transaction"
        />
      </MicroBlock>
    </MicroBlocks>
  </div>
</template>

<script>

import GenerationDetails from '../../../partials/generationDetails'
import MicroBlocks from '../../../partials/microBlocks'
import MicroBlock from '../../../partials/microBlock'
import PageHeader from '../../../components/PageHeader'
import TXListItem from '../../../partials/transactions/txListItem'

export default {
  name: 'AppGenerationDetails',
  components: {
    PageHeader,
    GenerationDetails,
    MicroBlocks,
    MicroBlock,
    TXListItem
  },
  data () {
    return {
      height: 0,
      prev: '',
      next: '',
      generation: null
    }
  },
  async asyncData ({ store, params }) {
    let generation = null
    let prev = null
    let next = null
    if (store.generations) {
      generation = store.generations.generations[params.generation]
      prev = store.state.generations.generations[params.generation - 1]
      next = store.state.generations.generations[params.generation + 1]
    } else {
      const generations = await store.dispatch('generations/getGenerationByRange', { start: params.generation - 1, end: params.generation + 1 })
      prev = generations[params.generation - 1]
      next = generations[params.generation + 1]
      generation = generations[params.generation]
    }
    const height = await store.dispatch('height')
    const current = generation.height
    const last = Number(Object.keys(store.state.generations.generations)[0])
    prev = last === current ? '' : `/generations/${prev.height}`
    if (next) {
      next = height === current ? '' : `/generations/${next.height}`
    }
    return { generation, prev, next, height }
  },
  methods: {
    getLast () {
      let heights = []
      for (let generation of this.$store.state.generations.generations) {
        heights.push(generation.height)
      }
      return heights.reverse()[0]
    }
  }
}
</script>
