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
        const data = JSON.parse(resp).payload
        if (data.tx) {
          this.updateTxList(data)
        }

        if (data.beneficiary) {
          this.updateGenList(data)
        }

        if (data.key_block_id) {
          this.updateMicroBlocks(data)
        }
      }
    },
    async updateTxList (tx) {
      this.$store.commit('transactions/setTransactions', [tx])
      this.$store.dispatch('generations/updateTx', tx)
      if (this.$store.state.height < tx.block_height) {
        await this.$store.dispatch('height')
      }
    },
    updateMicroBlocks (mb) {
      this.$store.dispatch('generations/updateMicroBlock', mb)
    },
    updateGenList (gen) {
      this.$store.commit('generations/setGenerations', [gen])
    }
  }
}
</script>
