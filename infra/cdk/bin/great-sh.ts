#!/usr/bin/env node
import * as cdk from 'aws-cdk-lib';
import { GreatShStack } from '../lib/great-sh-stack';

const app = new cdk.App();

new GreatShStack(app, 'GreatShStack', {
  env: {
    account: '756605216505',
    region: 'us-east-1',
  },
  hostedZoneId: app.node.tryGetContext('hostedZoneId') ?? 'PLACEHOLDER',
});

cdk.Tags.of(app).add('Project', 'GreatSh');
cdk.Tags.of(app).add('ManagedBy', 'CDK');
