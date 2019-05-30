<template>
  <div class="app-names">
    <PageHeader title="State Channels">
      <BreadCrumbs />
    </PageHeader>
    <ChannelList>
      <Channel
        v-for="(item, index) in channels"
        :key="index"
        :data="item"
      />
    </ChannelList>
    <LoadMoreButton @update="loadMore" />
  </div>
</template>

<script>
import ChannelList from '../../partials/channels/channelList'
import Channel from '../../partials/channels/channel'
import PageHeader from '../../components/PageHeader'
import BreadCrumbs from '../../components/breadCrumbs'
import LoadMoreButton from '../../components/loadMoreButton'
// import { mapState } from 'vuex'

export default {
  name: 'AppNames',
  components: {
    ChannelList,
    Channel,
    PageHeader,
    BreadCrumbs,
    LoadMoreButton
  },
  data () {
    return {
      page: 1
    }
  },
  computed: {
    channels () {
      return ['ch_pXcot855823NqFB7NUCFpew8hkiCfyvAz7xi58SGnPGJaJr8H', 'ch_Q6rUn61CviD9U2AiF3iQ7oxzqSAWXgpizQ3SzToDBMMXxiTaW', 'ch_bV5AYukdpuXC9VGyh4gw84YA7uruM1HXMqQ7ibBQeZgqsvTtS', 'ch_xymUJvByRJTDhXGjMj9oZXp8febkGEFUYi5cQsTc5H3SaLiiw', 'ch_uQMHczzyehpuXotp3Z66fskfAYuvJX3sBcaNM6tdEF3PVyu7s', 'ch_2Vi1o8WurSvCxLrp4FJdhftJrMDDxC17LF93sFoGbs4BxXQCxR', 'ch_214RAm1VhaqhevtsotLKz21RTTX9FJVwv4P2m8MM43MGnGRM5R', 'ch_AASLHAYDA42gcCsVM1W7G29Ywi4xd4bZrMth536fXrMxFaCys', 'ch_2HGk3yJmpU9vBvz922riaUEHbptek4P4WTcCqSirM8TjEWuqei', 'ch_2e4zjaTrAa91ZJarNkHDHViu1aBAnjVTeBzqPLhzxSF6KPaut7', 'ch_2BYjC8FEaC5x4o5Wi75gSPo3QPvtg5gx8jR1RDFqNNmPzGHHs8', 'ch_2kuAW2daTvHGHPzh5qvUA43reCJf3NFRY9Bob76bSyputKi5x7', 'ch_Gk6WBrRQL8sogFrYW8QCUhVJArUwPbKPYMbMdfrQPKqw6PPxB', 'ch_gotk22zRvqDmKXanexqiA7E15gPVyvfEwaxPL4S5srfYeX8g1', 'ch_2jgSXSmpB62dVDQgnqgzt5MCAq67CRm91pGWmQQKNSadKc73Wx', 'ch_2hUwsQcbrSjGu6ywcHPY4GAkVDdSWDxPDtrLPJLxFARmTtPLG2', 'ch_fw9PyowhGqLirDkKH5NZvVNKWhRkhfqW5Qk1QaUCWKHERvnyL', 'ch_XNWUPk5QfT4CDy7uXU3DUNNDSuMZspDjkxUZQqyiS5LbYRQ58', 'ch_HAcPmsPXT4FRWNzNQweH3Pk9qinTmc4QGgy1C4nL9vZm6gpNc', 'ch_2hHtDUipMafeKCc2NPZxTNKsHvWkFqvkvZUJd6yhnHYfUSTLsL', 'ch_aMMYtDGAKFg7zreqCVtXvtso92dBCBUsZ6pu1A3Zhv6JfHvwU', 'ch_2R5ZoVRnWiXtfaeNoUXMVEqJpV2a9dyMyPVXzkR6dHeF3CwTRb', 'ch_QVcBzKX4Ho6PccPSq7Vuau6C8rwBabFiB4BM6AzMRt716bJyy', 'ch_6MGmW6CPizKJ555iEfK1X8kGKFi8ERMpAP1u2pcxVTsw2AY2G', 'ch_283wxdyJAidtWqWHikvkPfNj89jrs88pxgtZwhFUEQFxoy1WSH', 'ch_2kk5P99v2xwD6KPXwhNq6bZhnJRNm14WvNsGmBVUhsLxggDwDi', 'ch_2Diky5Ub8UqjpEyUw5Vj77GXoyLRPstZuiPUTASrvfu7qSAjHL', 'ch_D3kXM6HPQ2sisJytfxgCsfK3KndjV3cHyGjVYPfR4YW1se5d9', 'ch_tWEwzLjSkm76nS33jGKiPL3Vf7CjauXhsi58d26BnkMa7fKCJ', 'ch_2dtVxgsTYS7EzDT2a4FRuKtBZFUQAERBAwpokoMsC7D8AUN2Fx', 'ch_2VZkwC2nAksiKAsGd51E9LGxGTaD3vesP1v1zVsBk8696oNdNH', 'ch_2oURn636Msn6VaMvWuaufqutAy9kiV8Kcd79N7BK2FT9H2W82o', 'ch_2eQYnupythn22jDTjnd32HhiezQ9y6WPg94Z2SsWoNhTUgmcob', 'ch_9AcrRUiYpJU7cXVVd6ipcufaxNegKxwVRpF91WqssLGrQfzhF', 'ch_QEiBhQ4poXLuczP226z4af9pnN5nPd8h3qDCrTYjSvRgVFx7k', 'ch_2LJYMSmMQdQ3XewxTVuvXNVGaCd7dxxpSK6fFS9rhyRLmHUM9r', 'ch_2PKUPw5qL3rYp8gPbwQbixcaryyMdH2wJpJT6HyNm5ZnH6abqw', 'ch_2apzirEPfHCvRQhs7YethN4PUcGY2JTYjf11D4gewfYFZbPjQt', 'ch_2AJzZHUQRVLr5utLwwEpvy81rA8VsxmoeXLYF4PU343toYqDaL', 'ch_3N64FNAwGDMtECRbxeqw4WXQ64kcwD7wnqQJAGSHvcbCFtCkg', 'ch_2cEq2dThV4B6JEn4iP4zU37m1dia5UjC9iam1ysecFUu2b5ArS', 'ch_n8pJokm1siJVvJpRKQVk5rg4eS13YpgzYtNCJU2Y8FtmMuUc5', 'ch_2FumwvxVpoopH5YxaT1QjfaY8PdvMvocSXCEuikX1M5kE3pGaA', 'ch_KidWCBwBaV9kXzTnvua81AJzQ5s2V8Zciug1CbHp4z44bpRV3', 'ch_2S1dXLx8SU1paFQiyevGH9jpkdBo7tZWVoiR1hSFyNbzpqYcf4', 'ch_JP22NWe19jPauZ67yNANC233oCgMnXpJ8JFMvRa29nnU4KSEb', 'ch_2iZp6rkDMePEqYFGcxJEgbA98Fvj7GDM7PKeW8pdbnNEcPyeRu', 'ch_2rW7V9nHhjFxP3RNyeDNJ3HhTKQgfPED4xpkGMawLFLErqCgHw', 'ch_2kmngR9dsD76GYGnbABzRA82JLMR8nqLs4cDhbp21UnhoWEpbg', 'ch_2S7Qtt8HwW8TqsbBGtuPYx8hKEsmXKd8EkkKnKDJ6bRpuRueuA', 'ch_2DL8zsy89AcmowkH94ZWk9Uax6Rvwy33bfCeDre5opiKHZedT1', 'ch_3cGyS13MavK6V74qeotG9JpqPuo5Zatch5NTjV8y66tXzb4xQ', 'ch_AcbmD62pTF1e8vG8ntuaDd4LEVYBN9ETq7UoZWo1msCDcvfjn', 'ch_25hBpiTb2truhtHrabXGM7JeWc6abdDhRAE4Xr7K3wKu7ecU99', 'ch_2dWCQNikerxVAcwrFN2e3uJTYb2KJGC9hpeqrcpqD13mT4DLap', 'ch_2pd8PoLUwGzDtV8rT9dhReF42Y3MCqBYeYpTdh26ZEAtBgJYGX', 'ch_2gfPrLDeaugqEsaq89QMqumo6B3DcpfZwwRDqHgTsu9bLUThgs', 'ch_J86vu2rW4q7KSQ3JCEXcC1SQ6C6GvBcqtggEH6FW8uds6q5bf', 'ch_VD7JwpFefencK7wMirR79xr6887ax76SkJtodtKmc7tLUj7Cm', 'ch_ZXEpqFZCupDRTg6jcbtysRZhcsqJXMHP6e5N9RRx7rVe1rmfs', 'ch_5Yf6HXXLupwjZRgHdhmzceJKtg6uAwywJdgcmkHQ5woxmT1GW']
    }
  },
  beforeMount () {
    this.loadMore()
  },
  methods: {
    loadMore () {
      this.$store.dispatch('names/getNames', { 'page': this.page, 'limit': 10 })
      this.page += 1
    }
  }
}
</script>
