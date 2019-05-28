<template>
  <div>
    <no-ssr>
      <div
        v-if="generations.length"
        class="generations-wrapper"
      >
        <PageHeader
          :is-main="false"
          title="Generations"
        />
        <Generations>
          <nuxt-link
            v-for="(generation, number) in generations.slice(0,5)"
            :key="number"
            :to="`/generations/${generation.height}`"
            class="generation-link"
          >
            <Generation
              :data="generation"
            />
          </nuxt-link>
        </Generations>
      </div>
      <div
        class="transactions-wrapper"
      >
        <PageHeader
          :is-main="false"
          title="Transactions"
        />
        <TxList>
          <TXListItem
            v-for="(transaction, index) in transactions.reverse().slice(0,5)"
            :key="index"
            :data="transaction"
          />
        </TxList>
      </div>
    </no-ssr>
  </div>
</template>
<script>
import Generations from '../partials/generations'
import Generation from '../partials/generation'
import TxList from '../partials/transactions/txList'
import TXListItem from '../partials/transactions/txListItem'
import PageHeader from '../components/PageHeader'
import { mapState } from 'vuex'

export default {
  name: 'AppDashboard',
  layout: 'default',
  components: {
    Generations,
    Generation,
    TxList,
    TXListItem,
    PageHeader
  },
  data () {
    return {
      data: {},
      currentGen: {},
      microBlocks: []
    }
  },
  computed: {
    ...mapState('generations', {
      generations (state) {
        return Object.values(state.generations).reverse()
      }
    }),
    ...mapState('transactions', {
      transactions (state) {
        return Object.values(state.transactions)
      }
    })
  },
  mounted () {
    const mdwWebsocket = new WebSocket(this.$store.state.wsUrl)

    mdwWebsocket.onopen = e => {
      mdwWebsocket.send('{"op":"subscribe", "payload": "key_blocks"}')
      mdwWebsocket.send('{"op":"subscribe", "payload": "micro_blocks"}')
      mdwWebsocket.send('{"op":"subscribe", "payload": "transactions"}')

      mdwWebsocket.onmessage = e => {
        this.getWsData(e.data)
      }
    }
  },
  methods: {
    getWsData (resp) {
      if (resp.includes('payload')) {
        this.data = JSON.parse(resp).payload
        if (this.data.tx) {
          this.updateTxList(this.data)
        }

        if (this.data.beneficiary) {
          this.updateGenList(this.data)
        }

        if (this.data.key_block_id) {
          this.updateMicroBlocks(this.data)
        }
      }
    },
    async updateTxList (tx) {
      this.$store.commit('transactions/setTransactions', [tx])
      if (this.$store.state.height < tx.block_height) {
        await this.$store.dispatch('height')
      }
    },
    updateMicroBlocks (mb) {
      this.microBlocks.push({ ...{}, ...mb })
    },
    updateGenList (gen) {
      if (!Object.keys(this.currentGen).length) {
        this.currentGen = { ...{}, ...gen, micro_blocks: [] }
        this.$store.commit('generations/setGenerations', [Object.assign({}, this.currentGen)])
      } else {
        // this.currentGen.micro_blocks = this.localTransactions.filter(tx => tx.block_height === this.currentGen.height)
        this.currentGen.micro_blocks = this.microBlocks
          .filter(mb => mb.prev_key_hash === this.currentGen.hash)
          .map(mb => {
            return { ...{}, ...mb, transactions: this.transactions.filter(tx => tx.block_hash === mb.hash) }
          })
        this.$store.commit('generations/setGenerations', [Object.assign({}, this.currentGen)])
        this.microBlocks = []
        this.currentGen = { ...{}, ...gen, micro_blocks: [] }
      }
    }
  }
}
</script>
