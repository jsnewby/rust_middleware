<template>
  <div class="transaction">
    <div class="transaction-main-info">
      <div class="transaction-main-info-inner">
        <nuxt-link :to="`/transactions/${transaction.hash}`">
          <div class="transaction-label">
            <LabelType
              :title="transaction.tx.type.replace(/([A-Z])/g, ' $1')"
              fill="red"
            />
          </div>
        </nuxt-link>
        <AppDefinition
          v-if="transaction.tx.time"
          title="Age"
        >
          <Age :time="transaction.tx.time" />
        </AppDefinition>
      </div>
      <div class="transaction-main-info-inner accounts">
        <AccountGroup>
          <Account
            v-if="transaction.tx.from_id"
            :value="transaction.tx.from_id"
            title="From Account"
            icon
          />
          <Account
            v-if="transaction.tx.channel_id"
            :value="transaction.tx.channel_id"
            title="channel id"
            icon
          />
        </AccountGroup>
        <Account
          v-if="transaction.tx.poi"
          :value="transaction.tx.poi"
          title="PoI"
          icon
        />
      </div>
    </div>
    <div class="transaction-type-info">
      <div class="transaction-type-info-item">
        <AppDefinition
          title="Block Height"
        >
          <nuxt-link :to="`/generations/${transaction.block_height}`">
            {{ transaction.block_height }}
          </nuxt-link>
        </AppDefinition>
        <AppDefinition
          v-if="transaction.tx.nonce"
          title="nonce"
        >
          {{ transaction.tx.nonce }}
        </AppDefinition>
      </div>
      <div class="transaction-type-info-item">
        <AppDefinition
          v-if="transaction.tx.fee"
          title="tx fee"
        >
          <FormatAeUnit :value="transaction.tx.fee" />
        </AppDefinition>
      </div>
    </div>
  </div>
</template>
<script>
import AppDefinition from '../../../components/appDefinition'
import FormatAeUnit from '../../../components/formatAeUnit'
import AccountGroup from '../../../components/accountGroup'
import Account from '../../../components/account'
import Age from '../../../components/age'
import LabelType from '../../../components/labelType'

export default {
  name: 'ChannelCloseSoloTx',
  components: {
    LabelType,
    AppDefinition,
    FormatAeUnit,
    AccountGroup,
    Account,
    Age
  },
  props: {
    transaction: {
      type: Object,
      required: true
    }
  }
}
</script>
