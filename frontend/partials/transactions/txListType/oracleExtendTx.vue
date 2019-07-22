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
        <Account
          v-if="transaction.tx.oracle_id"
          :value="transaction.tx.oracle_id"
          title="oracle"
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
          v-if="transaction.tx.fee"
          title="tx fee"
        >
          <FormatAeUnit
            :value="transaction.tx.fee"
          />
        </appdefinition>
      </div>
      <div class="transaction-type-info-item">
        <AppDefinition
          v-if="transaction.tx.oracle_ttl"
          title="oracle ttl type"
        >
          {{ transaction.tx.oracle_ttl.type }}
        </AppDefinition>
        <AppDefinition
          v-if="transaction.tx.oracle_ttl"
          title="oracle ttl value"
        >
          {{ transaction.tx.oracle_ttl.value }}
        </AppDefinition>
      </div>
    </div>
  </div>
</template>
<script>
import AppDefinition from '../../../components/appDefinition'
import FormatAeUnit from '../../../components/formatAeUnit'
import Account from '../../../components/account'
import Age from '../../../components/age'
import LabelType from '../../../components/labelType'

export default {
  name: 'OracleExtendTx',
  components: {
    LabelType,
    AppDefinition,
    FormatAeUnit,
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
