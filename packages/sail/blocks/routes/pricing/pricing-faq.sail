<controller>
  export default controller(() => {
    const faqs = [
      {
        id: 'switch',
        question: 'Can I switch plans later?',
        answer: 'Yes. Upgrade or downgrade from settings. Proration is automatic.',
      },
      {
        id: 'project',
        question: 'What counts as a project?',
        answer: 'One Sail app with its own theme overlay. Hobby is capped at one.',
      },
      {
        id: 'cut',
        question: 'Do you take a cut of usage?',
        answer: 'No. The compiler and L1 catalog are licensed per seat, not metered.',
      },
      {
        id: 'annual',
        question: 'Is annual billed up front?',
        answer: 'Yes. Studio annual is two months free versus month-to-month.',
      },
      {
        id: 'hobby',
        question: 'Can Hobby stay free forever?',
        answer: 'Yes, for one project. Move to Studio when you need Theme Studio or more seats.',
      },
    ]
    return { faqs }
  })
</controller>

<center>
  <column gap="4">
    <h2>Questions</h2>
    <for {faqs as item} key={item.id}>
      <accordion name="pricing-faq">
        <strong slot="trigger">{item.question}</strong>
        <p>{item.answer}</p>
      </accordion>
    </for>
  </column>
</center>
