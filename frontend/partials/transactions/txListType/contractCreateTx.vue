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
            v-if="transaction.tx.owner_id"
            :value="transaction.tx.owner_id"
            title="owner"
            icon
          />
          <Account
            v-if="transaction.contract_id"
            :value="transaction.contract_id"
            title="Contract"
            icon
          />
        </AccountGroup>
      </div>
    </div>
    <div class="transaction-type-info">
      <div class="transaction-type-info-item ">
        <AppDefinition
          v-if="transaction.tx.amount"
          title="Amount"
        >
          <FormatAeUnit
            :value="transaction.tx.amount"
          />
        </AppDefinition>
        <AppDefinition
          v-if="transaction.tx.deposit"
          title="Deposit"
        >
          <FormatAeUnit
            :value="transaction.tx.deposit"
          />
        </AppDefinition>
      </div>
      <div class="transaction-type-info-item">
        <AppDefinition
          v-if="transaction.tx.fee"
          title="tx fee"
        >
          <FormatAeUnit
            :value="transaction.tx.fee"
          />
        </AppDefinition>
        <AppDefinition
          v-if="transaction.tx.cost"
          title="tx cost"
        >
          <FormatAeUnit
            :value="transaction.tx.cost"
          />
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
  name: 'ContractCreateTx',
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
