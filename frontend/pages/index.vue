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
          <nuxt-link
            v-for="(transaction, index) in transactions.reverse().slice(0,5)"
            :key="index"
            :to="`/transactions/${transaction.hash}`"
          >
            <TXListItem
              :data="transaction"
            />
          </nuxt-link>
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
          this.$store.commit('transactions/setTransactions', [data])
          this.$store.dispatch('generations/updateTx', data)
        }
        if (data.beneficiary) {
          this.$store.commit('generations/setGenerations', [data])
          if (this.$store.state.height < data.height) {
            this.$store.commit('setHeight', data.height, { root: true })
          }
        }
        if (data.key_block_id) {
          this.$store.dispatch('generations/updateMicroBlock', data)
        }
      }
    }
  }
}
</script>
